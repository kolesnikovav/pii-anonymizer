FROM node:20-alpine

RUN apk add --no-cache docker-cli

WORKDIR /app

# Устанавливаем зависимости для HTTP-to-stdio MCP proxy
RUN npm init -y && \
    npm install express eventsource-parser uuid && \
    rm -rf /root/.npm

COPY server.js /app/server.js

EXPOSE 3001

ENTRYPOINT ["node", "server.js"]
