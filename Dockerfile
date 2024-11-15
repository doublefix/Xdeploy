FROM python:3.13.0-alpine3.20

WORKDIR /opt/xdeploy

COPY . .
RUN apk update \
    && pip install --no-cache-dir -r requirements.txt \
    && rm -rf /var/cache/apk/* \
    && rm -rf /root/.cache/pip
EXPOSE 5000

CMD ["gunicorn", "--bind", "0.0.0.0:5005", "run:app"]
