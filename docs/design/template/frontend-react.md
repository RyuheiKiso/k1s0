# frontend-react テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/frontend/react/{service_name}/
├── .k1s0/
│   └── manifest.json
├── package.json.tera
├── tsconfig.json
├── vite.config.ts
├── README.md
├── public/
│   └── index.html
├── src/
│   ├── main.tsx.tera
│   ├── App.tsx
│   ├── domain/
│   │   ├── entities/
│   │   └── repositories/
│   ├── application/
│   │   ├── services/
│   │   └── usecases/
│   └── presentation/
│       ├── components/
│       ├── pages/
│       └── hooks/
└── deploy/
    ├── base/
    └── overlays/
```

## package.json.tera

```json
{
  "name": "{{ feature_name }}",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint src --ext .ts,.tsx",
    "test": "vitest"
  },
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "react-router-dom": "^6.23.0"
  },
  "devDependencies": {
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.4.0",
    "vite": "^5.2.0",
    "vitest": "^1.6.0"
  }
}
```
