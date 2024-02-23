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

use std::{fs::File, io::Write, path::Path, time::Duration};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, Subscriber},
    layer::SubscriberExt,
    EnvFilter,
};

pub fn init_logger(
    log_file: &Path,
    log_level: &Level,
) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let mut file = File::options().append(true).create(true).open(log_file)?;
    if file.metadata()?.created()?.elapsed()? > Duration::from_secs(1) {
        file.write_all(b"\n")?;
    }
    let (file_writer, file_guard) = tracing_appender::non_blocking(file);
    let subscriber = Subscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(log_level.as_str())),
        )
        .finish()
        .with(tracing_journald::layer()?)
        .with(fmt::layer().with_writer(file_writer));
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(file_guard)
}
