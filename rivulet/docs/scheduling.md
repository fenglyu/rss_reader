# Automatic Feed Updates

Rivulet supports automatic background feed updates through two approaches:

1. **Built-in Daemon** (recommended) - Cross-platform, no configuration needed
2. **System Scheduler** - Uses OS-native tools (cron, systemd, Task Scheduler)

---

## Option 1: Built-in Daemon

The simplest approach - works on all platforms without system configuration.

### Start the Daemon

```bash
# Start with default settings (updates every 1 hour)
rivulet daemon start

# Custom interval
rivulet daemon start --interval 6h    # Every 6 hours
rivulet daemon start --interval 30m   # Every 30 minutes
rivulet daemon start --interval 1d    # Once per day

# With logging to file
rivulet daemon start --interval 1h --log ~/.local/log/rivulet.log

# Skip initial update on start
rivulet daemon start --no-initial-update

# Run in foreground (useful for debugging)
rivulet daemon start --foreground
```

### Check Status

```bash
rivulet daemon status
# Output: Daemon is running (PID: 12345)
# Output: Daemon is not running
```

### Stop the Daemon

```bash
rivulet daemon stop
```

### Auto-start on Login

#### macOS

Create `~/Library/LaunchAgents/com.rivulet.daemon.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rivulet.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/rivulet</string>
        <string>daemon</string>
        <string>start</string>
        <string>--foreground</string>
        <string>--interval</string>
        <string>1h</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/rivulet-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/rivulet-daemon.log</string>
</dict>
</plist>
```

Load it:
```bash
launchctl load ~/Library/LaunchAgents/com.rivulet.daemon.plist
```

#### Linux (systemd)

Create `~/.config/systemd/user/rivulet-daemon.service`:

```ini
[Unit]
Description=Rivulet RSS Daemon

[Service]
ExecStart=/path/to/rivulet daemon start --foreground --interval 1h
Restart=on-failure

[Install]
WantedBy=default.target
```

Enable it:
```bash
systemctl --user enable --now rivulet-daemon.service
```

#### Windows

Add to startup via Task Scheduler or create a shortcut in the Startup folder:
```
%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
```

Create a shortcut pointing to:
```
C:\path\to\rivulet.exe daemon start --interval 1h
```

---

## Option 2: System Scheduler (cron/systemd/Task Scheduler)

For users who prefer native OS tools or need more control.

### macOS (launchd)

Create `~/Library/LaunchAgents/com.rivulet.update.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rivulet.update</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/rivulet</string>
        <string>update</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>8</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/rivulet-update.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/rivulet-update.log</string>
</dict>
</plist>
```

Load it:
```bash
launchctl load ~/Library/LaunchAgents/com.rivulet.update.plist
```

Unload:
```bash
launchctl unload ~/Library/LaunchAgents/com.rivulet.update.plist
```

### Ubuntu/Linux (systemd timer)

**~/.config/systemd/user/rivulet-update.service**:
```ini
[Unit]
Description=Update Rivulet RSS feeds

[Service]
Type=oneshot
ExecStart=/path/to/rivulet update
```

**~/.config/systemd/user/rivulet-update.timer**:
```ini
[Unit]
Description=Daily Rivulet RSS update

[Timer]
OnCalendar=daily
# Or for hourly: OnCalendar=hourly
# Or for specific time: OnCalendar=*-*-* 08:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable:
```bash
systemctl --user daemon-reload
systemctl --user enable --now rivulet-update.timer
```

Check status:
```bash
systemctl --user status rivulet-update.timer
systemctl --user list-timers
```

### Ubuntu/Linux (cron)

Simple alternative to systemd:

```bash
crontab -e
```

Add one of these lines:

```bash
# Every hour
0 * * * * /path/to/rivulet update >> /tmp/rivulet-update.log 2>&1

# Every 6 hours
0 */6 * * * /path/to/rivulet update >> /tmp/rivulet-update.log 2>&1

# Daily at 8am
0 8 * * * /path/to/rivulet update >> /tmp/rivulet-update.log 2>&1

# Twice daily (8am and 8pm)
0 8,20 * * * /path/to/rivulet update >> /tmp/rivulet-update.log 2>&1
```

### Windows (Task Scheduler)

#### PowerShell

```powershell
$action = New-ScheduledTaskAction -Execute "C:\path\to\rivulet.exe" -Argument "update"
$trigger = New-ScheduledTaskTrigger -Daily -At 8am
# Or hourly:
# $trigger = New-ScheduledTaskTrigger -Once -At (Get-Date) -RepetitionInterval (New-TimeSpan -Hours 1)
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -DontStopOnIdleEnd
Register-ScheduledTask -TaskName "RivuletUpdate" -Action $action -Trigger $trigger -Settings $settings
```

Remove:
```powershell
Unregister-ScheduledTask -TaskName "RivuletUpdate" -Confirm:$false
```

#### GUI Method

1. Open Task Scheduler (`taskschd.msc`)
2. Click "Create Basic Task..."
3. Name: "Rivulet Update"
4. Trigger: Daily (or your preference)
5. Action: Start a program
6. Program: `C:\path\to\rivulet.exe`
7. Arguments: `update`
8. Finish

---

## Comparison

| Feature | Built-in Daemon | System Scheduler |
|---------|-----------------|------------------|
| Cross-platform | Yes | Requires per-OS config |
| Easy setup | `rivulet daemon start` | Manual config files |
| Precise intervals | 30m, 1h, 6h, 1d | Depends on scheduler |
| Resource usage | Minimal (sleeps between updates) | Only runs when triggered |
| Survives reboot | Needs autostart setup | Native persistence |
| Best for | Most users | Power users, servers |

---

## Troubleshooting

### Daemon won't start
- Check if another instance is running: `rivulet daemon status`
- Check logs if using `--log` option
- Try running in foreground: `rivulet daemon start --foreground`

### Updates not running
- Verify rivulet path is correct and executable
- Check log files for errors
- Ensure network connectivity

### Too many/few updates
- Adjust `--interval` for daemon
- Modify timer/cron schedule for system scheduler
