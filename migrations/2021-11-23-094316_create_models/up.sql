CREATE TABLE workspaces (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    premium BOOLEAN NOT NULL,
    admin BOOLEAN NOT NULL,
    default_hourly_rate INTEGER NOT NULL,
    default_currency TEXT NOT NULL,
    only_admins_may_create_projects BOOLEAN NOT NULL,
    only_admins_see_billable_rates BOOLEAN NOT NULL,
    rounding INTEGER NOT NULL,
    rounding_minutes INTEGER NOT NULL,
    at TEXT NOT NULL,
    logo_url TEXT,

    user_id INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    wid INTEGER NOT NULL,
    cid INTEGER,
    active BOOLEAN NOT NULL,
    is_private BOOLEAN NOT NULL,
    template BOOLEAN,
    template_id INTEGER,
    billable BOOLEAN,
    auto_estimates BOOLEAN,
    estimated_hours INTEGER,
    at TEXT NOT NULL,
    color TEXT NOT NULL,
    rate REAL,
    created_at TEXT NOT NULL
);

CREATE TABLE clients (
    id INTEGER PRIMARY KEY,
    wid INTEGER NOT NULL,
    name TEXT NOT NULL,
    at TEXT NOT NULL,

    user_id INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    api_token INTEGER NOT NULL,
    default_wid_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    fullname TEXT NOT NULL,
    jquery_timeofday_format TEXT NOT NULL,
    jquery_date_format TEXT NOT NULL,
    timeofday_format TEXT NOT NULL,
    date_format TEXT NOT NULL,
    store_start_and_stop_time BOOLEAN NOT NULL,
    beginning_of_week INTEGER NOT NULL,
    language TEXT NOT NULL,
    image_url TEXT NOT NULL,
    sidebar_piechart BOOLEAN NOT NULL,
    at TEXT NOT NULL,
    send_product_emails BOOLEAN NOT NULL,
    send_weekly_report BOOLEAN NOT NULL,
    send_timer_notifications BOOLEAN NOT NULL,
    openid_enabled BOOLEAN NOT NULL,
    timezone TEXT NOT NULL,

    FOREIGN KEY(default_wid_id) REFERENCES workspace(id)
);


CREATE TABLE time_entrys (
    id INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    wid INTEGER,
    pid INTEGER,
    billable BOOLEAN,
    start TEXT NOT NULL,
    stop TEXT,
    duration INTEGER NOT NULL,
    created_with TEXT,
    duronly BOOLEAN,
    at TEXT,

    FOREIGN KEY(pid) REFERENCES projects(id),
    FOREIGN KEY(wid) REFERENCES workspaces(id)
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    wid INTEGER NOT NULL,
    user_id INTEGER NOT NULL,

    FOREIGN KEY(wid) REFERENCES workspace(id),
    FOREIGN KEY(user_id) REFERENCES users(id),

    UNIQUE(wid, name)
);

CREATE TABLE time_entry_tag_join (
    time_entry_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,

    FOREIGN KEY(tag_id) REFERENCES tags(id),
    FOREIGN KEY(time_entry_id) REFERENCES time_entrys(id),
    PRIMARY KEY (tag_id, time_entry_id),
    UNIQUE(time_entry_id, tag_id)
);
