# 🚀 Deployment Guide

This guide explains how to deploy the **URL Shortener (ZTM)** project to popular platforms like **Railway**, **Fly.io**, and **DigitalOcean**.

---

## 🧩 Prerequisites
Before deploying, make sure you have:
- Node.js (v18+)
- Git installed
- A MongoDB database (e.g., MongoDB Atlas)
- Environment variables configured locally and ready for production

Required environment variables:
```bash
PORT=5000
MONGO_URI=your-mongodb-connection-string
BASE_URL=https://your-production-domain.com