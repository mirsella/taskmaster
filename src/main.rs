/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 11:35:35 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod parsing_conf;
use parsing_conf::{ Config, get_config };

fn main() {
    println!("Hello, world!");
	let conf: Config = get_config().unwrap();
	
	println!{"{:#?}", conf};
}
