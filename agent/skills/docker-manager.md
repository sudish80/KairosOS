# Docker Manager Skill

## Description
Manage Docker containers, images, volumes, and compose stacks.

## Triggers
- User asks about containers
- Docker daemon issues
- Container deployment requests

## Actions

### list_containers(all)
List running (or all) containers

### list_images()
List Docker images

### ps()
Show running containers with resource usage

### logs(container, lines)
View container logs

### start(container)
Start a container

### stop(container)
Stop a container

### exec(container, command)
Execute a command in a running container

### compose_up(path)
Start a docker-compose stack

### compose_down(path)
Stop a docker-compose stack

### prune()
Clean up unused containers, images, and volumes

## Dependencies
- Docker daemon
- docker-compose (optional)
