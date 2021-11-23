CREATE TABLE workspaces (
id INTEGER PRIMARY KEY,
name TEXT NOT NULL,
premium BOOLEAN NOT NULL,
admin BOOLEAN NOT NULL,
default_hourly_rate INTEGER NOT NULL,
default_currency TEXT NOT NULL,
only_admins_may_create_projects: BOOLEAN NOT NULL,
only_admins_see_billable_rates: BOOLEAN NOT NULL,
rounding: INTEGER NOT NULL,
rounding_minutes: INTEGER NOT NULL,
at TEXT NOT NULL,
logo_url TEXT,
)

