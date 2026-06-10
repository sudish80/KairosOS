# Cron Manager Skill

## Description
Schedule and manage automated tasks. Creates cron jobs and systemd timers through natural language.

## Triggers
- User wants to schedule recurring tasks
- Daily system maintenance
- Automated backup requests

## Actions

### schedule(name, command, schedule)
Create a scheduled task:
- `name`: task identifier
- `command`: what to run
- `schedule`: natural language schedule ("every day at 2am", "every hour", "every Monday")

### list()
Show all scheduled tasks

### remove(name)
Remove a scheduled task

### pause(name)
Pause a scheduled task without removing it

### resume(name)
Resume a paused task

### run_now(name)
Execute a scheduled task immediately

### logs(name)
View execution history of a task

## Example
User: "Schedule a backup every night at 3am"
Agent: *creates a systemd timer that runs the backup script daily at 3am*

## Dependencies
- systemd timer units
- crond (fallback)
