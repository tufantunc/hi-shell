# Scheduled Build Worker

This Cloudflare Worker triggers daily builds to keep PostHog stats up-to-date on the website.

## Setup Instructions

### 1. Create Deploy Hook

1. Go to Cloudflare Dashboard → Pages → hi-shell-web
2. Navigate to **Settings** → **Builds** → **Deploy hooks**
3. Click **Add deploy hook**
4. Name: `scheduled-build`, Branch: `main`
5. Copy the generated URL

### 2. Create the Worker

1. Go to Cloudflare Dashboard → Workers & Pages
2. Click **Create** → **Create Worker**
3. Name it `hi-shell-scheduled-build`
4. Click **Deploy**
5. Click **Edit code** and paste the contents of `worker.js`
6. Click **Deploy**

### 3. Add Environment Variable

1. Go to Worker **Settings** → **Variables and Secrets**
2. Add variable:
   - Name: `DEPLOY_HOOK_URL`
   - Value: (paste your deploy hook URL from step 1)

### 4. Add Cron Trigger

1. Go to Worker **Settings** → **Triggers**
2. Click **Add Cron Trigger**
3. Enter cron expression: `0 5 * * *` (runs daily at 05:00 UTC)
4. Click **Add**

## Environment Variables for PostHog

Add these to your Cloudflare Pages project:

1. Go to Pages → hi-shell-web → **Settings** → **Environment variables**
2. Add:
   - `POSTHOG_API_KEY`: Your PostHog personal API key (from PostHog → Settings → Personal API keys)
   - `POSTHOG_PROJECT_ID`: Your PostHog project ID (visible in the URL when viewing your project)

## Cron Expression Reference

| Expression      | Description                    |
|-----------------|--------------------------------|
| `0 5 * * *`     | Every day at 05:00 UTC         |
| `0 */12 * * *`  | Every 12 hours                 |
| `0 9 * * 1-5`   | Weekdays at 09:00 UTC          |
| `0 0 * * 0`     | Every Sunday at midnight UTC   |
