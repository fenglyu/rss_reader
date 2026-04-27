#!/usr/bin/env node
/**
 * Spark UI Scraper — Playwright automation for extracting Spark Web UI evidence.
 *
 * Usage:
 *   npx playwright test spark-ui-scraper.mjs  (via Playwright test runner)
 *   node spark-ui-scraper.mjs <base-url> [--action sql|stages|executors|environment|sql-detail]
 *
 * Examples:
 *   node spark-ui-scraper.mjs https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 --action sql
 *   node spark-ui-scraper.mjs https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 --action sql-detail --query-id 3
 *   node spark-ui-scraper.mjs https://badge-megarm.tiktok-row.net/proxy/application_1773129570997_1179136 --action all
 */

import { chromium } from 'playwright';

const TIMEOUT = 30_000;

// ---------------------------------------------------------------------------
// CLI arg parsing
// ---------------------------------------------------------------------------
function parseArgs() {
  const args = process.argv.slice(2);
  const opts = { action: 'all', queryId: null, baseUrl: null };
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--action' && args[i + 1]) {
      opts.action = args[i + 1];
      i++;
    } else if (args[i] === '--query-id' && args[i + 1]) {
      opts.queryId = args[i + 1];
      i++;
    } else if (!args[i].startsWith('--')) {
      opts.baseUrl = args[i];
    }
  }
  if (!opts.baseUrl) {
    console.error('Usage: node spark-ui-scraper.mjs <base-url> [--action sql|stages|executors|environment|sql-detail|all] [--query-id N]');
    process.exit(1);
  }
  // Strip trailing slash
  opts.baseUrl = opts.baseUrl.replace(/\/+$/, '');
  return opts;
}

// ---------------------------------------------------------------------------
// Page helpers
// ---------------------------------------------------------------------------

async function extractTable(page, selector) {
  return page.evaluate((sel) => {
    const table = document.querySelector(sel);
    if (!table) return null;
    const rows = [];
    for (const tr of table.querySelectorAll('tr')) {
      const cells = [];
      for (const td of tr.querySelectorAll('th, td')) {
        cells.push(td.innerText.trim());
      }
      if (cells.length) rows.push(cells);
    }
    return rows;
  }, selector);
}

async function extractAllTables(page) {
  return page.evaluate(() => {
    const tables = document.querySelectorAll('table');
    const result = [];
    for (const table of tables) {
      const rows = [];
      for (const tr of table.querySelectorAll('tr')) {
        const cells = [];
        for (const td of tr.querySelectorAll('th, td')) {
          cells.push(td.innerText.trim());
        }
        if (cells.length) rows.push(cells);
      }
      if (rows.length) result.push(rows);
    }
    return result;
  });
}

function formatTable(rows) {
  if (!rows || !rows.length) return '  (no data)\n';
  // Compute column widths
  const widths = rows[0].map((_, ci) =>
    Math.max(...rows.map((r) => (r[ci] || '').length))
  );
  return rows
    .map((r) =>
      '  ' + r.map((c, ci) => (c || '').padEnd(widths[ci])).join(' | ')
    )
    .join('\n') + '\n';
}

// ---------------------------------------------------------------------------
// Tab scrapers
// ---------------------------------------------------------------------------

async function scrapeSQL(page, baseUrl) {
  console.log('\n=== SQL Tab ===');
  await page.goto(`${baseUrl}/SQL/`, { waitUntil: 'domcontentloaded', timeout: TIMEOUT });
  await page.waitForTimeout(2000); // Allow dynamic content to load

  // Extract Running Queries table
  console.log('\n--- Running Queries ---');
  const runningTable = await extractTable(page, '#running-table, table.table:first-of-type');
  if (runningTable && runningTable.length > 1) {
    console.log(formatTable(runningTable));
    // Extract query links from running queries
    const queryLinks = await page.evaluate(() => {
      const links = [];
      const rows = document.querySelectorAll('#running-table tr, table.table:first-of-type tr');
      for (const row of rows) {
        const link = row.querySelector('a[href*="execution"]');
        const desc = row.querySelector('td:nth-child(2), td:nth-child(3)');
        if (link) {
          links.push({
            href: link.href,
            text: link.innerText.trim(),
            description: desc ? desc.innerText.trim().substring(0, 120) : ''
          });
        }
      }
      return links;
    });
    if (queryLinks.length) {
      console.log('\nQuery links found:');
      for (const ql of queryLinks) {
        console.log(`  - ${ql.text}: ${ql.href}`);
        if (ql.description) console.log(`    Description: ${ql.description}`);
      }
    }
  } else {
    console.log('  (no running queries)');
  }

  // Extract Completed Queries table
  console.log('\n--- Completed Queries ---');
  const completedTable = await extractTable(page, '#completed-table, table.table:nth-of-type(2)');
  if (completedTable && completedTable.length > 1) {
    // Show first 20 rows max
    const rows = completedTable.slice(0, 21);
    console.log(formatTable(rows));
    if (completedTable.length > 21) {
      console.log(`  ... (${completedTable.length - 1} total queries, showing first 20)`);
    }
  } else {
    console.log('  (no completed queries)');
  }

  return { runningTable, completedTable };
}

async function scrapeSQLDetail(page, baseUrl, queryId) {
  console.log(`\n=== SQL Query Detail (id=${queryId}) ===`);
  await page.goto(`${baseUrl}/SQL/execution/?id=${queryId}`, {
    waitUntil: 'domcontentloaded',
    timeout: TIMEOUT
  });
  await page.waitForTimeout(2000);

  // Click "Details" button to expand physical plan
  const detailsButton = await page.$('button:has-text("Details"), a:has-text("Details"), span:has-text("Details")');
  if (detailsButton) {
    await detailsButton.click();
    await page.waitForTimeout(1000);
    console.log('  (Details section expanded)');
  }

  // Also try clicking any collapsible plan sections
  const expandButtons = await page.$$('[data-toggle="collapse"], .collapse-trigger, button[aria-expanded="false"]');
  for (const btn of expandButtons) {
    try {
      await btn.click();
      await page.waitForTimeout(300);
    } catch {
      // ignore click failures on non-interactive elements
    }
  }

  // Extract the physical plan text
  const physicalPlan = await page.evaluate(() => {
    // Look for the physical plan in multiple possible locations
    const selectors = [
      '#physical-plan-details',
      '.physical-plan',
      'pre:has-text("== Physical Plan ==")',
      '#plan-viz-metadata-size',
      '[id*="plan"]',
      'pre',
      '.plan-details',
      '#planViz-metadata',
    ];
    for (const sel of selectors) {
      const el = document.querySelector(sel);
      if (el && el.innerText.includes('Physical Plan')) {
        return el.innerText;
      }
    }
    // Fallback: find any element containing "Physical Plan"
    const all = document.querySelectorAll('*');
    for (const el of all) {
      if (el.children.length === 0 && el.innerText && el.innerText.includes('== Physical Plan ==')) {
        return el.innerText;
      }
    }
    // Last resort: look in any <pre> or <code> block
    for (const pre of document.querySelectorAll('pre, code')) {
      const text = pre.innerText;
      if (text.length > 100) return text;
    }
    return null;
  });

  if (physicalPlan) {
    console.log('\n--- Physical Plan ---');
    console.log(physicalPlan);
  } else {
    console.log('\n--- Physical Plan ---');
    console.log('  (not found — plan may require manual expansion)');
  }

  // Extract query metrics tables
  console.log('\n--- Query Metrics ---');
  const tables = await extractAllTables(page);
  for (const t of tables) {
    if (t.length > 0) {
      console.log(formatTable(t));
      console.log('');
    }
  }

  // Get the page title / query description
  const title = await page.evaluate(() => {
    const h4 = document.querySelector('h4, .page-title');
    return h4 ? h4.innerText.trim() : document.title;
  });
  console.log(`\nQuery title: ${title}`);

  return { physicalPlan, tables, title };
}

async function scrapeStages(page, baseUrl) {
  console.log('\n=== Stages Tab ===');
  await page.goto(`${baseUrl}/stages/`, { waitUntil: 'domcontentloaded', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  console.log('\n--- Active Stages ---');
  const activeTable = await extractTable(page, '#active-table, table:first-of-type');
  console.log(formatTable(activeTable));

  console.log('\n--- Completed Stages ---');
  const completedTable = await extractTable(page, '#completed-table, table:nth-of-type(2)');
  if (completedTable) {
    const rows = completedTable.slice(0, 21);
    console.log(formatTable(rows));
  }

  console.log('\n--- Failed Stages ---');
  const failedTable = await extractTable(page, '#failed-table');
  console.log(formatTable(failedTable));

  return { activeTable, completedTable, failedTable };
}

async function scrapeExecutors(page, baseUrl) {
  console.log('\n=== Executors Tab ===');
  await page.goto(`${baseUrl}/executors/`, { waitUntil: 'domcontentloaded', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  const tables = await extractAllTables(page);
  for (const t of tables) {
    console.log(formatTable(t));
    console.log('');
  }

  return { tables };
}

async function scrapeEnvironment(page, baseUrl) {
  console.log('\n=== Environment Tab ===');
  await page.goto(`${baseUrl}/environment/`, { waitUntil: 'domcontentloaded', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  // Extract Spark Properties specifically
  console.log('\n--- Spark Properties ---');
  const sparkProps = await page.evaluate(() => {
    const tables = document.querySelectorAll('table');
    for (const table of tables) {
      const heading = table.previousElementSibling;
      if (heading && heading.innerText.includes('Spark Properties')) {
        const rows = [];
        for (const tr of table.querySelectorAll('tr')) {
          const cells = [];
          for (const td of tr.querySelectorAll('th, td')) {
            cells.push(td.innerText.trim());
          }
          if (cells.length) rows.push(cells);
        }
        return rows;
      }
    }
    // Fallback: get all tables
    return null;
  });

  if (sparkProps) {
    console.log(formatTable(sparkProps));
  } else {
    const tables = await extractAllTables(page);
    for (const t of tables) {
      console.log(formatTable(t));
      console.log('');
    }
  }

  return { sparkProps };
}

async function scrapeJobs(page, baseUrl) {
  console.log('\n=== Jobs Tab ===');
  await page.goto(`${baseUrl}/jobs/`, { waitUntil: 'domcontentloaded', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  const tables = await extractAllTables(page);
  for (const t of tables) {
    console.log(formatTable(t));
    console.log('');
  }

  return { tables };
}

// ---------------------------------------------------------------------------
// SQL flow: find longest-running query and drill into its detail
// ---------------------------------------------------------------------------

async function sqlDebugFlow(page, baseUrl) {
  console.log('\n========================================');
  console.log('  Spark SQL Debug Flow — Automated');
  console.log('========================================');

  // Step 1: Go to SQL tab and find running queries
  const sqlData = await scrapeSQL(page, baseUrl);

  // Step 2: Find the longest running query link
  await page.goto(`${baseUrl}/SQL/`, { waitUntil: 'domcontentloaded', timeout: TIMEOUT });
  await page.waitForTimeout(2000);

  const longestQuery = await page.evaluate(() => {
    // Look for running queries first, then completed
    const tables = document.querySelectorAll('table');
    let bestLink = null;
    let bestDuration = 0;

    for (const table of tables) {
      const rows = table.querySelectorAll('tr');
      for (const row of rows) {
        const cells = row.querySelectorAll('td');
        for (const cell of cells) {
          // Find duration cells (contain time formats like "1.2 h", "45 min", "30 s")
          const text = cell.innerText;
          let durationMs = 0;
          const hourMatch = text.match(/([\d.]+)\s*h/);
          const minMatch = text.match(/([\d.]+)\s*min/);
          const secMatch = text.match(/([\d.]+)\s*s(?!e)/);
          if (hourMatch) durationMs += parseFloat(hourMatch[1]) * 3600000;
          if (minMatch) durationMs += parseFloat(minMatch[1]) * 60000;
          if (secMatch) durationMs += parseFloat(secMatch[1]) * 1000;

          if (durationMs > bestDuration) {
            const link = row.querySelector('a[href*="execution"]');
            if (link) {
              bestDuration = durationMs;
              const idMatch = link.href.match(/id=(\d+)/);
              bestLink = {
                href: link.href,
                id: idMatch ? idMatch[1] : null,
                duration: text.trim(),
                durationMs
              };
            }
          }
        }
      }
    }
    return bestLink;
  });

  if (longestQuery && longestQuery.id) {
    console.log(`\nLongest-running query: ID ${longestQuery.id} (${longestQuery.duration})`);
    console.log(`Navigating to detail page...`);
    await scrapeSQLDetail(page, baseUrl, longestQuery.id);
  } else {
    console.log('\nNo query links found to drill into.');
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    ignoreHTTPSErrors: true,
    viewport: { width: 1920, height: 1080 }
  });
  const page = await context.newPage();

  try {
    switch (opts.action) {
      case 'sql':
        await scrapeSQL(page, opts.baseUrl);
        break;
      case 'sql-detail':
        if (!opts.queryId) {
          console.error('--query-id is required for sql-detail action');
          process.exit(1);
        }
        await scrapeSQLDetail(page, opts.baseUrl, opts.queryId);
        break;
      case 'sql-debug':
        await sqlDebugFlow(page, opts.baseUrl);
        break;
      case 'stages':
        await scrapeStages(page, opts.baseUrl);
        break;
      case 'executors':
        await scrapeExecutors(page, opts.baseUrl);
        break;
      case 'environment':
        await scrapeEnvironment(page, opts.baseUrl);
        break;
      case 'jobs':
        await scrapeJobs(page, opts.baseUrl);
        break;
      case 'all': {
        await scrapeJobs(page, opts.baseUrl);
        await scrapeStages(page, opts.baseUrl);
        await scrapeExecutors(page, opts.baseUrl);
        await scrapeSQL(page, opts.baseUrl);
        await scrapeEnvironment(page, opts.baseUrl);
        break;
      }
      default:
        console.error(`Unknown action: ${opts.action}`);
        process.exit(1);
    }
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error('Error:', err.message);
  process.exit(1);
});
