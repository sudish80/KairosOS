class Config:
    socket_path: str = "/var/run/kairos/skill-evolver.sock"
    log_level: str = "INFO"

    def __init__(self, socket_path=None, log_level=None):
        import os
        self.socket_path = socket_path or os.getenv("KAIROS_SKILL_EVOLVER_ENDPOINT", self.socket_path)
        self.log_level = log_level or os.getenv("LOG_LEVEL", self.log_level)
