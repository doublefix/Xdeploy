FROM python:3.13.0-alpine3.20

WORKDIR /opt/xdeploy

COPY . .
RUN apk update \
    && apk add --no-cache openssh \
    && ssh-keygen -t rsa -b 4096 -f /etc/ssh/ssh_rsa_key -N "" \
    && ssh-keygen -t ecdsa -b 521 -f /etc/ssh/ssh_ecdsa_key -N "" \
    && ssh-keygen -t ed25519 -f /etc/ssh/ssh_ed25519_key -N "" \
    && pip install --no-cache-dir -r requirements.txt \
    && rm -rf /var/cache/apk/* \
    && rm -rf /root/.cache/pip
EXPOSE 5005

CMD ["gunicorn", "--bind", "0.0.0.0:5005", "run:app"]
