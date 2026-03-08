# Dockerfile for AAMN Dashboard
FROM node:20-alpine AS build

WORKDIR /app
COPY ./dashboard/package*.json ./
RUN npm install --legacy-peer-deps

COPY ./dashboard/ ./
RUN npm run build

# Stage 2: Serve using Nginx
FROM nginx:stable-alpine
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
