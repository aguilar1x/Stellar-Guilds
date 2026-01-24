# Backend API

NestJS backend service.

## Setup

```bash
cd backend
npm install
npm run start:dev
```

### Environment Variables

Copy .env.example to .env and update values.

### API Documentation

Swagger UI available at:

http://localhost:3000/docs

### Development

- Node.js >= 18
- NestJS
- TypeScript

```yaml
Copy code
```
---

## 8️⃣ Testing checklist (acceptance criteria)

Run these manually:

```bash
npm run build        # TypeScript compiles
npm run start:dev    # Server starts
```
### Verify

- [x] Server starts on port 3000
- [x] GET / returns "Hello World!"
- [x] /docs loads Swagger
- [x] No TypeScript errors
- [x] Env vars accessible via process.env