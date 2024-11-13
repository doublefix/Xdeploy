FROM python:3.13.0-alpine3.20

WORKDIR /app

COPY . .

RUN apk update \
    && apk add --no-cache gcc libffi-dev musl-dev \
    && pip install --no-cache-dir -r requirements.txt \
    && apk del gcc libffi-dev musl-dev

EXPOSE 5000

ENV FLASK_APP=app.py
ENV FLASK_RUN_HOST=0.0.0.0

CMD ["flask", "run"]