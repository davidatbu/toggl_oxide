table! {
    clients (id) {
        id -> Nullable<Integer>,
        wid -> Integer,
        name -> Text,
        at -> Text,
        user_id -> Integer,
    }
}

table! {
    projects (id) {
        id -> Nullable<Integer>,
        name -> Text,
        wid -> Integer,
        cid -> Nullable<Integer>,
        active -> Bool,
        is_private -> Bool,
        template -> Nullable<Bool>,
        template_id -> Nullable<Integer>,
        billable -> Nullable<Bool>,
        auto_estimates -> Nullable<Bool>,
        estimated_hours -> Nullable<Integer>,
        at -> Text,
        color -> Text,
        rate -> Nullable<Float>,
        created_at -> Text,
    }
}

table! {
    tags (id) {
        id -> Nullable<Integer>,
        name -> Text,
        wid -> Integer,
        user_id -> Integer,
    }
}

table! {
    time_entry_tag_join (time_entry_id, tag_id) {
        time_entry_id -> Integer,
        tag_id -> Integer,
    }
}

table! {
    time_entrys (id) {
        id -> Nullable<Integer>,
        description -> Text,
        wid -> Nullable<Integer>,
        pid -> Nullable<Integer>,
        billable -> Nullable<Bool>,
        start -> Text,
        stop -> Nullable<Text>,
        duration -> Integer,
        created_with -> Nullable<Text>,
        duronly -> Nullable<Bool>,
        at -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Nullable<Integer>,
        api_token -> Integer,
        default_wid_id -> Integer,
        email -> Text,
        fullname -> Text,
        jquery_timeofday_format -> Text,
        jquery_date_format -> Text,
        timeofday_format -> Text,
        date_format -> Text,
        store_start_and_stop_time -> Bool,
        beginning_of_week -> Integer,
        language -> Text,
        image_url -> Text,
        sidebar_piechart -> Bool,
        at -> Text,
        send_product_emails -> Bool,
        send_weekly_report -> Bool,
        send_timer_notifications -> Bool,
        openid_enabled -> Bool,
        timezone -> Text,
    }
}

table! {
    workspaces (id) {
        id -> Nullable<Integer>,
        name -> Text,
        premium -> Bool,
        admin -> Bool,
        default_hourly_rate -> Integer,
        default_currency -> Text,
        only_admins_may_create_projects -> Bool,
        only_admins_see_billable_rates -> Bool,
        rounding -> Integer,
        rounding_minutes -> Integer,
        at -> Text,
        logo_url -> Nullable<Text>,
        user_id -> Integer,
    }
}

joinable!(clients -> users (user_id));
joinable!(tags -> users (user_id));
joinable!(time_entry_tag_join -> tags (tag_id));
joinable!(time_entry_tag_join -> time_entrys (time_entry_id));
joinable!(time_entrys -> projects (pid));
joinable!(time_entrys -> workspaces (wid));
joinable!(workspaces -> users (user_id));

allow_tables_to_appear_in_same_query!(
    clients,
    projects,
    tags,
    time_entry_tag_join,
    time_entrys,
    users,
    workspaces,
);
