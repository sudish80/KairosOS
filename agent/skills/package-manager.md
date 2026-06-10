# Package Manager Skill

## Description
Manage software packages across multiple package managers. Handles apt, pacman, dnf, and pip.

## Triggers
- User asks to install, update, or remove software
- Dependency conflict resolution
- System update requests

## Actions

### install(package, manager)
Install a package using the appropriate package manager:
- `package`: package name(s)
- `manager`: auto | apt | pacman | dnf | pip | npm
- Auto-detects the system's package manager

### remove(package, manager)
Remove an installed package

### update_all()
Update all packages on the system:
- System packages via native package manager
- Python packages via pip
- Node packages via npm

### search(query)
Search for packages matching the query

### list_outdated()
List all packages with available updates

### autoremove()
Remove unnecessary dependencies and clean package cache

## Example
User: "Install nginx and enable HTTPS"
Agent: *installs nginx, configures SSL, opens firewall port, enables service*

## Dependencies
- Package manager (apt/pacman/dnf)
- Sudo access
- pip, npm (optional)
