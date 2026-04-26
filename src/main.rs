use anyhow::{anyhow, Context, Result};
use dotenvy::from_filename;
use odbc_api::{buffers::TextRowSet, ConnectionOptions, Cursor, Environment, ResultSetMetadata};
use regex::Regex;
use rusqlite::{params_from_iter, types::ValueRef, Connection as SqliteConnection};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

mod tui;
mod db;

use db::*;

#[derive(Clone, Debug)]
struct RunProgressSnapshot {
    total_items: usize,
    done_items: usize,
    current_item: String,
    current_item_rows_done: usize,
    current_item_rows_total: usize,
    status: String,
}

impl Default for RunProgressSnapshot {
    fn default() -> Self {
        Self {
            total_items: 0,
            done_items: 0,
            current_item: String::new(),
            current_item_rows_done: 0,
            current_item_rows_total: 0,
            status: "idle".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct RunProgressEvent {
    total_items: Option<usize>,
    done_items: Option<usize>,
    current_item: Option<String>,
    current_item_rows_done: Option<usize>,
    current_item_rows_total: Option<usize>,
    status: Option<String>,
}

impl RunProgressSnapshot {
    fn apply_event(&mut self, ev: RunProgressEvent) {
        if let Some(v) = ev.total_items {
            self.total_items = v;
        }
        if let Some(v) = ev.done_items {
            self.done_items = v;
        }
        if let Some(v) = ev.current_item {
            self.current_item = v;
        }
        if let Some(v) = ev.current_item_rows_done {
            self.current_item_rows_done = v;
        }
        if let Some(v) = ev.current_item_rows_total {
            self.current_item_rows_total = v;
        }
        if let Some(v) = ev.status {
            self.status = v;
        }
    }
}


include!("app/cli_config_types.rs");
include!("app/cli_argument_parser.rs");
include!("app/env_persistence.rs");
include!("app/sql_text_helpers.rs");
include!("app/object_query_export.rs");
include!("app/table_page_export.rs");
include!("app/export_runner.rs");
include!("app/import_runner.rs");
include!("app/program_entry.rs");
