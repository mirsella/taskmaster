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
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, registry, reload, util::SubscriberInitExt, EnvFilter,
    Registry,
};
use tui_logger::tracing_subscriber_layer;

pub fn init_logger(
    log_file: &Path,
) -> Result<reload::Handle<EnvFilter, Registry>, Box<dyn std::error::Error>> {
    let mut file = File::options().append(true).create(true).open(log_file)?;
    if file.metadata()?.created()?.elapsed()? > Duration::from_secs(1) {
        file.write_all(b"\n")?;
    }

    tui_logger::set_default_level(log::LevelFilter::Trace);

    let (filter_layer, filter_handle) =
        reload::Layer::new(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")));
    registry()
        .with(filter_layer)
        .with(tracing_subscriber_layer())
        .with(tracing_journald::layer()?)
        .with(layer().with_writer(file))
        .init();
    Ok(filter_handle)
}
