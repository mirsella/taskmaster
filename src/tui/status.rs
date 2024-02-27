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
    style::Color,
    widgets::{Cell, Row, Table},
};
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
        let statuss = self.childs.iter().map(|c| c.status).collect::<Vec<_>>();
        let mut vec: Vec<(Status, i32)> = Vec::new();
        for status in statuss {
            if let Some(v) = vec.iter_mut().find(|x| x.0.eq_ignore_instant(&status)) {
                v.1 += 1;
                if v.0.get_instant() < status.get_instant() {
                    v.0 = status;
                }
            } else {
                vec.push((status, 1));
            }
        }

        let mut lines = vec![];
        for (status, count) in vec {
            lines.push(Row::new([
                Cell::from(self.name.clone()),
                Cell::from(status.to_string())
                    .style(status.color(&self.valid_exit_codes, self.stop_signal as i32)),
                Cell::from(format!("{count}/{}", self.childs.len())),
                Cell::from(format!("{:?}", status.get_instant().elapsed())),
            ]));
        }

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
