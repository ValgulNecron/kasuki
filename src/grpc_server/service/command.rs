// Proto module contains the protobuf definitions for the shard service
pub(crate) mod proto {
    // Include the protobuf definitions for the shard service
    tonic::include_proto!("command");
    // FILE_DESCRIPTOR_SET is a constant byte array that contains the file descriptor set for the shard service
    pub(crate) const COMMAND_FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("command_descriptor");
}

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::grpc_server::command_list::{Arg, Command, CommandItem, SubCommand, SubCommandGroup};
use crate::grpc_server::service::command::proto::{CommandListRequest, CommandListResponse};
use crate::grpc_server::service::command::proto::command_service_server::{CommandService, CommandServiceServer};

pub struct CommandServices {
    pub command_list: Arc<Vec<CommandItem>>,
}

#[tonic::async_trait]
impl CommandService for CommandServices {
    async fn command_list(
        &self,
        _request: Request<CommandListRequest>,
    ) -> Result<Response<CommandListResponse>, Status> {
        let cmd_list = &self.command_list.clone();
        let cm_count = cmd_list.len();
        let mut commands = Vec::new();
        let mut sub_commands = Vec::new();
        let mut sub_command_groups = Vec::new();
        for cmd in cmd_list.iter() {
            match cmd {
                CommandItem::Command(c) => {
                    commands.push(c.into());
                }
                CommandItem::Subcommand(s) => {
                    sub_commands.push(s.into());
                }
                CommandItem::SubcommandGroup(sg) => {
                    sub_command_groups.push(sg.into());
                }
            }
        }
        let response = CommandListResponse {
            command_count: cm_count as i64,
            commands,
            sub_commands,
            sub_command_groups,
        };
        Ok(Response::new(response))
    }
}


impl From<&Command> for proto::Command {
    fn from(command: &Command) -> Self {
        proto::Command {
            name: command.name.clone(),
            description: command.desc.clone(),
            args: command
                .args
                .clone()
                .into_iter()
                .map(|arg| (&arg).into())
                .collect(),
        }
    }
}

impl From<&Arg> for proto::Arg {
    fn from(arg: &Arg) -> Self {
        proto::Arg {
            name: arg.name.clone(),
            description: arg.desc.clone(),
            required: arg.required,
            choices: arg.choices.clone(),
        }
    }
}

impl From<&SubCommand> for proto::SubCommand {
    fn from(subcommand: &SubCommand) -> Self {
        proto::SubCommand {
            name: subcommand.name.clone(),
            description: subcommand.desc.clone(),
            commands: subcommand
                .commands
                .clone()
                .into_iter()
                .map(|commands| (&commands).into())
                .collect(),
        }
    }
}

impl From<&SubCommandGroup> for proto::SubCommandGroup {
    fn from(subcommand_group: &SubCommandGroup) -> Self {
        proto::SubCommandGroup {
            name: subcommand_group.name.clone(),
            description: subcommand_group.desc.clone(),
            sub_commands: subcommand_group
                .subcommands
                .clone()
                .into_iter()
                .map(|subcommands| (&subcommands).into())
                .collect(),
            commands: subcommand_group
                .commands
                .clone()
                .into_iter()
                .map(|commands| (&commands).into())
                .collect(),
        }
    }
}

pub fn get_command_server(command_service: CommandServices) -> CommandServiceServer<CommandServices> {
    CommandServiceServer::new(command_service)
}