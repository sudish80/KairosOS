# Filesystem Manager Skill

## Description
Manage filesystems, disks, partitions, and storage. Handle mounts, permissions, and disk health.

## Triggers
- User reports disk full or storage issues
- New disk/partition detected
- Permission problems
- Backup requests

## Actions

### disk_usage(path)
Show disk usage for a path or mount point

### list_disks()
List all block devices with partition info

### mount(device, mountpoint, options)
Mount a filesystem

### umount(mountpoint)
Unmount a filesystem

### check_disk(device)
Run filesystem check (fsck)

### find_large_files(path, size)
Find files larger than size in path (e.g., "100M")

### analyze_usage(path)
Analyze disk usage by directory (du)

### repair_permissions(path, mode, owner)
Fix file permissions and ownership

### create_snapshot(source, dest)
Create a filesystem snapshot (using rsync)

## Example
User: "My disk is almost full, clean it up"
Agent: *analyzes usage, finds large log files, suggests cleanup, removes temp files*

## Dependencies
- df, du, lsblk
- mount, umount
- fsck
- rsync
