from flask import Flask, make_response, request, jsonify
import os
from app.routes import routes
from app.task import TASKS_DIR

app = Flask(__name__)

os.makedirs(TASKS_DIR, exist_ok=True)
app.register_blueprint(routes)

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
