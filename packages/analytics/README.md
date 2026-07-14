# `@dash/analytics`

Client-side analytics event tracking with batched flushing.

## Overview

Queues analytics events (page views, identify, track) and flushes them in batches to a configurable endpoint.

## Usage

```ts
import { createAnalyticsClient } from "@dash/analytics";

const analytics = createAnalyticsClient({
  endpoint: "https://analytics.dash.sh/events",
  flushInterval: 5000,
});

analytics.track("package_installed", { name: "react", version: "18.2.0" });
analytics.pageView("/packages/react");
```
