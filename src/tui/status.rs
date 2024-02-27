/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   status.rs                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/26 17:22:15 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/27 13:48:20 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use crate::program::{child::Status, Program};
use ratatui::{
    style::{Color, Style},
    widgets::{Cell, Row, Table},
};
use std::{borrow::BorrowMut, collections::HashMap, mem, time::Instant};

pub fn status(programs: &[Program]) -> Table {
    let mut rows = vec![Row::new(vec!["Name", "Status", "Processes", "Last update"])];
    rows.push(Row::new(vec!["╺━━━━━╸"]));
    for prog in programs {
        let mut status_rows = prog.status();
        if !status_rows.is_empty() {
            status_rows.push(Row::new(vec!["╺━━━━━╸"]));
        }
        rows.extend(status_rows.clone());
    }
    Table::new(rows, &[])
}

impl Program {
    pub fn status(&self) -> Vec<Row> {
        let same_instant = Instant::now();
        let statuss = self
            .childs
            .iter()
            .map(|c| {
                c.status.to_owned().set_instant(same_instant);
                c
            })
            .collect::<Vec<_>>();
        let mut map = HashMap::new();
        for status in statuss {
            let key = status.status.to_owned();
            let value = map.entry(key).or_insert(0);
            *value += 1;
        }

        let mut lines = vec![];
        if lines.is_empty() {
            lines.push(Row::new([
                self.name.clone(),
                "No processes".to_string(),
                "0".to_string(),
                "".to_string(),
            ]))
        }
        lines
    }
}

impl Status {
    pub fn color(&self, valid_codes: &[i32], valid_signal: i32) -> Color {
        match self {
            Status::Stopped(_) => Color::Blue,
            Status::Starting(_) => Color::Cyan,
            Status::Terminating(_) => Color::Yellow,
            Status::Running(_) => Color::Green,
            Status::Finished(_, code) => {
                if valid_codes.contains(code) {
                    Color::Gray
                } else {
                    Color::Red
                }
            }
            Status::Terminated(_, signal) => {
                if signal == &valid_signal {
                    Color::Gray
                } else {
                    Color::Red
                }
            }
        }
    }
}
