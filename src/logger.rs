/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 16:53:03 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 09:49:47 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::{env, fs::File, io::Write, path::Path};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, Subscriber},
    layer::SubscriberExt,
};

pub fn init_logger(
    log_file: &Path,
    log_level: &Level,
) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let log_level = env::var("LOG_LEVEL")
        .map(|str| str.parse::<Level>().unwrap_or(*log_level))
        .unwrap_or(*log_level);
    let mut file = File::options().append(true).create(true).open(log_file)?;
    file.write_all(b"\n")?;
    let (file_writer, file_guard) = tracing_appender::non_blocking(file);

    let subscriber = Subscriber::builder()
        .with_max_level(log_level)
        .finish()
        .with(tracing_journald::layer()?)
        .with(fmt::layer().with_writer(file_writer));
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(file_guard)
}
