pub static SENTRY_RELEASES: [&str; 3] = [
    "backend@c49de132b1ee681aa0f8cab2c118c693dcdb79dc",
    "javascript@c49de132b1ee681aa0f8cab2c118c693dcdb79dc",
    "snuba@aa4dc63a5ff733cf7b2fe3bd15b8f7a18d21e569",
];

pub static SENTRY_ENVIRONMENTS: [&str; 3] = ["production", "staging", "development"];

pub static SENTRY_TRANSACTIONS: [&str; 50] = [
    "/api/0/internal/rpc/{service_name}/{method_name}/",
    "getsentry.billing.tasks.usagebuffer.flush_usage_buffer",
    "/api/0/organizations/{organization_id_or_slug}/artifactbundle/assemble/",
    "/api/0/organizations/{organization_id_or_slug}/chunk-upload/",
    "/api/0/projects/{organization_id_or_slug}/{project_id_or_slug}/releases/{version}/files/",
    "sentry.dynamic_sampling.boost_low_volume_projects_of_org",
    "/api/0/internal/integration-proxy/",
    "/api/0/organizations/{organization_id_or_slug}/events/",
    "sentry.tasks.assemble.assemble_artifacts",
    "/api/0/projects/{organization_id_or_slug}/{project_id_or_slug}/overview/",
    "sentry.tasks.process_buffer.process_incr",
    "/api/0/organizations/{organization_id_or_slug}/issues/",
    "/api/0/organizations/{organization_id_or_slug}/repos/",
    "/api/0/organizations/{organization_id_or_slug}/issues|groups/{issue_id}/",
    "/api/0/organizations/{organization_id_or_slug}/releases/stats/",
    "/extensions/jira/issue-updated/",
    "/api/0/organizations/{organization_id_or_slug}/events-stats/",
    "/api/0/projects/{organization_id_or_slug}/{project_id_or_slug}/releases/{version}/",
    "sentry.tasks.files.delete_unreferenced_blobs",
    "/api/0/projects/{organization_id_or_slug}/{project_id_or_slug}/releases/",
    "/api/0/organizations/{organization_id_or_slug}/tags/",
    "/api/0/organizations/{organization_id_or_slug}/users/",
    "/api/0/organizations/{organization_id_or_slug}/sessions/",
    "/api/0/organizations/{organization_id_or_slug}/issues|groups/{issue_id}/tags/",
    "/api/0/organizations/{organization_id_or_slug}/replay-count/",
    "/api/0/organizations/{organization_id_or_slug}/projects/",
    "sentry.integrations.slack.tasks.send_activity_notifications_to_slack_threads",
    "sentry.tasks.assemble.assemble_dif",
    "/api/0/organizations/{organization_id_or_slug}/issues|groups/{issue_id}/events/{event_id}/",
    "sentry.hybridcloud.tasks.deliver_webhooks.drain_mailbox",
    "/api/0/projects/{organization_id_or_slug}/{project_id_or_slug}/events/{event_id}/attachments/",
    "/api/0/organizations/{organization_id_or_slug}/releases/{version}/deploys/",
    "/api/0/organizations/{organization_id_or_slug}/stats_v2/",
    "/api/0/organizations/{organization_slug}/broadcasts/",
    "/api/0/organizations/{organization_id_or_slug}/releases/{version}/",
    "/extensions/github/webhook/",
    "/api/0/organizations/{organization_id_or_slug}/prompts-activity/",
    "getsentry.tasks.stats.sync_organization",
    "/api/0/organizations/{organization_id_or_slug}/issues-count/",
    "/extensions/jira-server/issue-updated/{token}/",
    "sentry.debug_files.tasks.backfill_artifact_bundle_db_indexing",
    "sentry.tasks.email.send_email",
    "sentry.tasks.relay.invalidate_project_config",
    "/api/0/organizations/{organization_id_or_slug}/monitors/{monitor_id_or_slug}/checkins/",
    "/api/0/organizations/{organization_id_or_slug}/issues|groups/{issue_id}/tags/{key}/values/",
    "/api/0/organizations/{organization_id_or_slug}/flags/logs/",
    "/extensions/vsts/issue-updated/",
    "/api/0/organizations/{organization_id_or_slug}/events/{project_id_or_slug}:{event_id}/",
    "/api/0/organizations/{organization_id_or_slug}/recent-searches/",
    "/api/0/organizations/{organization_id_or_slug}/group-search-views/",
];

pub static HTTP_METHODS: [&str; 3] = ["GET", "POST", "PUT"];

pub static ROOT_OPS: [&str; 5] = [
    "http.server",
    "queue.task.celery",
    "celery.task",
    "default",
    "task",
];

pub static SPAN_OPS: [&str; 25] = [
    "tasks.post_process.run_post_process_job.pipeline.duration",
    "db",
    "middleware.django",
    "db.redis",
    "features.has",
    "function",
    "cache.get",
    "feature.flagpole.batch_has",
    "flagpole.option_retrieval_and_parsing",
    "jobs.duration",
    "queue.process",
    "queue.task.celery",
    "default",
    "http.client",
    "http.server",
    "bulk_record",
    "transaction.atomic",
    "other",
    "process_profile.track_outcome",
    "tasks.post_process_group.project_get_from_cache",
    "tasks.post_process_group.fetch_buffered_group_stats",
    "tasks.post_process_group.process_rules",
    "events.saved_to_post_processed",
    "tasks.post_process_group.rule_processor_callbacks",
    "events.time-to-post-process",
];

pub static BROWSER_NAMES: [&str; 5] = ["Chrome", "Firefox", "Safari", "Edge", "Opera"];

pub static SENTRY_SDKS: [&str; 3] = ["sentry.javascript.browser", "sentry.python", "sentry.rust"];

pub static SENTRY_PLATFORMS: [&str; 4] = ["javascript", "python", "rust", "other"];

pub static THREAD_NAMES: [&str; 1] = ["ThreadPoolExecutor-68_0"];
