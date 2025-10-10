# 🚀 Deployment Guide

This guide explains how to deploy the **URL Shortener (ZTM)** project to popular platforms like **Railway**, **Fly.io**, and **DigitalOcean**.

---

## 🧩 Prerequisites

Before deploying, make sure you have:

- Rust (latest stable version)
- Cargo (Rust package manager)
- Git installed
- A PostgreSQL or SQLite database (depending on configuration)
- Environment variables configured locally and ready for production

Required environment variables:

```bash
DATABASE_URL=your-database-connection-string
PORT=8000
BASE_URL=https://your-production-domain.com
RUST_LOG=info