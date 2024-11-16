import logging
import os
from logging.handlers import TimedRotatingFileHandler

os.makedirs("logs", exist_ok=True)

logger = logging.getLogger("flask_app")
logger.setLevel(logging.INFO)

console_handler = logging.StreamHandler()
console_handler.setLevel(logging.INFO)
log_file_handler = TimedRotatingFileHandler(
    "logs/xdeploy.log", when="midnight", interval=1, backupCount=7
)
log_file_handler.setLevel(logging.INFO)

log_format = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
console_handler.setFormatter(log_format)
log_file_handler.setFormatter(log_format)

logger.addHandler(console_handler)
logger.addHandler(log_file_handler)
logger.info("Logger setup complete.")
