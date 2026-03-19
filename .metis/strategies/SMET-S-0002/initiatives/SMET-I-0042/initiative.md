---
id: notifications-and-mobile-control
level: initiative
title: "Notifications and Mobile Control"
short_code: "SMET-I-0042"
created_at: 2026-03-17T19:56:54.624969+00:00
updated_at: 2026-03-17T19:56:54.624969+00:00
parent: SMET-S-0002
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: SMET-S-0002
initiative_id: notifications-and-mobile-control
---

# Notifications and Mobile Control Initiative

## Context

The core value proposition of the remote operations layer is that users can step away from their desk and still manage AI sessions. Push notifications and a mobile-optimized UI are what make this real. When a session needs approval or fails, the user must be notified immediately and be able to act from their phone in as few taps as possible.

This initiative covers: push notification delivery (FCM/APNs integration), configurable notification preferences, the notification inbox/queue in the dashboard, and the mobile-first responsive design for all key actions. The mobile experience is not an afterthought — layout, tap target size, and information density must be designed for mobile first.

**Pre-requisites**: SMET-I-0038, SMET-I-0039, SMET-I-0040, SMET-I-0041 (notification triggers come from session events).

**Components touched**: Control Service (notification generation, push delivery), Control Dashboard (mobile-first layouts, notification inbox, notification settings).

## Goals & Non-Goals

**Goals:**
- Push notification delivery when sessions need input, complete, fail, or become stuck (FCM/APNs)
- User-configurable notification types (which events trigger notifications)
- Urgent notifications (approval requests) routed distinctly from informational (completion)
- Notification taps navigate directly to the relevant session and action — no extra steps
- Notification preview shows what action is needed so user knows if immediate response is required
- Central notification inbox/queue: all sessions needing attention, showing why each needs it
- Mobile-first responsive layouts for all key dashboard views (phone, tablet, desktop)
- Long outputs and event streams remain readable on small screens
- Session controls visible while scrolling (sticky/fixed positioning)
- High-priority actions reachable with minimal taps from the inbox

**Non-Goals:**
- Native mobile apps (web-responsive PWA is sufficient for MVP)
- SMS or email notifications (push only for MVP)
- Notification analytics or delivery tracking beyond basic status

## Detailed Design

### Notification Triggers (Control Service)
Events that generate notifications:
- `ApprovalRequest` emitted → push "Session needs your input" (urgent priority)
- Session state → `failed` → push "Session failed" (high priority)
- Session state → `completed` → push "Session completed" (normal priority)
- Session stuck (no events for > N minutes while `running`) → push "Session may be stuck" (high priority)
- User-configurable: each notification type can be enabled/disabled per user

### Push Delivery (Control Service)
- Integrate with FCM (Android/web) and APNs (iOS) via a push provider library
- Store device tokens when users register from a device
- Notification payload: session\_id, event\_type, summary text (what action is needed)
- Deep link in notification: `control-web.example.com/sessions/{id}` → dashboard navigates to session and opens relevant tab

### Notification Inbox (Control Dashboard)
- `GET /notifications` — list of pending notifications for the user (unread first)
- Each notification shows: session title, machine, why attention is needed, timestamp
- Tap/click → navigate directly to session detail with appropriate tab focused (Prompts tab for approvals)
- Mark as read / dismiss
- Badge count on inbox icon shows unread count

### Mobile-First Layout Principles
- All primary controls in thumb-reach zone (bottom 60% of screen on phone)
- Session list cards: title, state badge, machine name, elapsed — no overflow
- Session detail: Pending Prompts tab shown first on mobile if there are pending approvals
- Approval response buttons: large tap targets (min 44px), full-width on phone
- Output stream: monospace, font size readable without pinch-zoom (min 13px on mobile)
- Sticky session action bar (stop/inject) visible while scrolling output
- Bottom navigation bar for: Sessions | Inbox | Machines | Settings

## Multi-Tenancy Notes

### Schema Changes
- `notifications` table: `user_id` foreign key — all notifications are owned by a specific user
- `device_tokens` table: `user_id` foreign key — device tokens are per-user (a developer registers their phone to their account)
- **MVP**: `user_id=1` for all records; device token registration still functions correctly

### API Scoping
- `GET /notifications`: `WHERE user_id = :current_user` — users only see their own notification inbox
- `POST /users/me/devices`: registers device token under `current_user` — device tokens never cross users
- Push delivery: notification dispatch looks up device tokens `WHERE user_id = notification.user_id` — a user's approval request only pings their devices

### Future Multi-User Behavior
- When a second user registers, they get a separate notification inbox and separate device tokens — no change to the notification logic needed
- Team-level notifications (e.g., notify all team members when a shared machine goes offline) would be a new notification type added on top; not in MVP
- Notification preferences are per-user: `notification_preferences` table with `user_id` foreign key — seeded with defaults for MVP user

## Alternatives Considered

- **Email notifications instead of push**: too slow for time-sensitive approvals; push chosen for latency; email could be added later as opt-in
- **In-app polling instead of push**: works only when app is open; misses the core use case (phone in pocket); rejected
- **SMS via Twilio**: higher infrastructure cost and complexity for MVP; push covers the same use case; rejected for MVP

## Implementation Plan

1. Define notification data model (id, user\_id, session\_id, type, priority, payload, read\_at)
2. Integrate push provider SDK in Control Service (FCM + APNs)
3. Implement device token registration endpoint (`POST /users/me/devices`)
4. Implement notification generation logic triggered by session events
5. Implement notification inbox API (`GET /notifications`, `POST /notifications/{id}/read`)
6. Build notification inbox view in dashboard
7. Implement mobile-first responsive layout for session list, session detail, and approval cards
8. Build bottom navigation bar for mobile
9. Test push delivery end-to-end: approval event → FCM push → tap → dashboard opens approval card
10. Test notification preferences: disable completion notifications, verify approval still fires