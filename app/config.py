import os
from flask.cli import load_dotenv

load_dotenv()


class Config:
    """Base configuration with default values"""

    TASKS_DIR = os.getenv("TASKS_DIR", "tasks")
    MAX_TASKS = int(os.getenv("MAX_TASKS", 3))
    DEBUG = False
    TESTING = False


class DevelopmentConfig(Config):
    """Development configuration"""

    DEBUG = True


class TestingConfig(Config):
    """Testing configuration"""

    TESTING = True


class ProductionConfig(Config):
    """Production configuration"""

    TESTING = False
    DEBUG = False
