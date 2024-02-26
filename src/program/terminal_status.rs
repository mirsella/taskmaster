/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   terminal_status.rs                                 :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/26 17:22:15 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/26 17:28:03 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use crate::program::{child::Status, Program};

use ratatui::{style::{Color, Style}, widgets::{Cell, Row}};

impl Program {
	/// Terminal status
	fn status_running(&self, force: bool) -> Option<Row> {
		let since = self.childs.iter().max_by_key(|x| x.last_update());
		let running: usize = self.childs.iter()
			.filter(|&c| {
				matches!(c.status, Status::Running(_))
			}).count();
		if running == 0 && !force{
			return None;
		}
		let since_str = match since {
			Some(c) => format!("{:?}", c.last_update().elapsed()),
			None => "Unknown".to_string(),
		};
		let status_str = format!("{running}/{}", self.childs.len());

		Some(Row::new(vec![
			Cell::from(self.name.clone()),
			Cell::from("Running".to_string()).style(Style::new().fg(Color::Green)),
			Cell::from(status_str),
			Cell::from(since_str)
		]))
	}

	fn status_starting(&self) -> Option<Row> {
		let since = self.childs.iter().max_by_key(|x| x.last_update());
		let running: usize = self.childs.iter()
			.filter(|&c| {
				matches!(c.status, Status::Starting(_))
			}).count();
		if running == 0 {
			return None;
		}
		let since_str = match since {
			Some(c) => format!("{:?}", c.last_update().elapsed()),
			None => "Unknown".to_string(),
		};
		let status_str = format!("{running}/{}", self.childs.len());

		Some(Row::new(vec![
			Cell::from(self.name.clone()),
			Cell::from("Starting".to_string()).style(Style::new().fg(Color::Yellow)),
			Cell::from(status_str),
			Cell::from(since_str)
		]))
	}

	fn status_finished(&self) -> Option<Row> {
		let since = self.childs.iter().max_by_key(|x| x.last_update());
		let running: usize = self.childs.iter()
			.filter(|&c| {
				matches!(c.status, Status::Finished(_, _))
			}).count();
		if running == 0 {
			return None;
		}
		let since_str = match since {
			Some(c) => format!("{:?}", c.last_update().elapsed()),
			None => "Unknown".to_string(),
		};
		let status_str = format!("{running}/{}", self.childs.len());

		Some(Row::new(vec![
			Cell::from(self.name.clone()),
			Cell::from("Finished".to_string()).style(Style::new().fg(Color::White)),
			Cell::from(status_str),
			Cell::from(since_str)
		]))
	}
	
	fn status_terminating(&self) -> Option<Row> {
		let since = self.childs.iter().max_by_key(|x| x.last_update());
		let running: usize = self.childs.iter()
			.filter(|&c| {
				matches!(c.status, Status::Terminating(_))
			}).count();
		if running == 0 {
			return None;
		}
		let since_str = match since {
			Some(c) => format!("{:?}", c.last_update().elapsed()),
			None => "Unknown".to_string(),
		};
		let status_str = format!("{running}/{}", self.childs.len());

		Some(Row::new(vec![
			Cell::from(self.name.clone()),
			Cell::from("Terminating".to_string()).style(Style::new().fg(Color::Blue)),
			Cell::from(status_str),
			Cell::from(since_str)
		]))
	}

	fn status_stopped(&self) -> Option<Row> {
		let since = self.childs.iter().max_by_key(|x| x.last_update());
		let running: usize = self.childs.iter()
			.filter(|&c| {
				matches!(c.status, Status::Stopped(_))
			}).count();
		if running == 0 {
			return None;
		}
		let since_str = match since {
			Some(c) => format!("{:?}", c.last_update().elapsed()),
			None => "Unknown".to_string(),
		};
		let status_str = format!("{running}/{}", self.childs.len());

		Some(Row::new(vec![
			Cell::from(self.name.clone()),
			Cell::from("Stopped".to_string()).style(Style::new().fg(Color::Gray)),
			Cell::from(status_str),
			Cell::from(since_str)
		]))
	}
	
	pub fn status(&self) -> Vec<Row> {
		let running = self.status_running(false);
		let terminating = self.status_terminating();
		let starting = self.status_starting();
		let finished = self.status_finished();
		let stopped = self.status_stopped();
		let mut res_lines: Vec<Row> = vec![];

		for line in [running, starting, terminating, stopped, finished].into_iter() {
			match line {
				Some(l) => res_lines.push(l),
				None => {},
			}
		}
		
		if res_lines.len() == 0 {
			// Safe because status_running always return a value if force == true
			res_lines.push(self.status_running(true).unwrap());
		}
		
		res_lines
	}
}