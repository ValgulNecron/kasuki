use crate::command::registry::{
	ContextType, GroupDef, InstallType, ParentCommand, PermissionType,
};

// ─── Subcommand parents (no handler, just grouping metadata) ─────────────────

inventory::submit!(&ParentCommand {
	name: "user",
	desc: "General purpose commands for user.",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::BotDm, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild, InstallType::User],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "ai",
	desc: "Command from the AI module",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::BotDm, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild, InstallType::User],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "bot",
	desc: "Command to get information about the bot.",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::BotDm, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild, InstallType::User],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "music",
	desc: "Command from the Music module",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild],
	install_contexts: &[InstallType::Guild],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "steam",
	desc: "Steam command from the GAME module.",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::BotDm, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild, InstallType::User],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "server",
	desc: "General purpose commands for server.",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "vn",
	desc: "Get info of a VN",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::BotDm, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild, InstallType::User],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "random_anime",
	desc: "Command from the ANIME module",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::BotDm, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild, InstallType::User],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "random_hanime",
	desc: "Command from the ANIME module",
	nsfw: true,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::BotDm, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild, InstallType::User],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "levels",
	desc: "Command to get level of user, and statistic.",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild],
	groups: &[],
});

inventory::submit!(&ParentCommand {
	name: "minigame",
	desc: "Commands for playing minigames and managing your inventory.",
	nsfw: false,
	permissions: &[],
	contexts: &[ContextType::Guild, ContextType::PrivateChannel],
	install_contexts: &[InstallType::Guild],
	groups: &[],
});

// ─── Subcommand group parent (admin with nested groups) ─────────────────────

inventory::submit!(&ParentCommand {
	name: "admin",
	desc: "Bot configuration configuration for admin only.",
	nsfw: false,
	permissions: &[PermissionType::Administrator],
	contexts: &[ContextType::Guild],
	install_contexts: &[InstallType::Guild],
	groups: &[
		GroupDef {
			name: "anilist",
			desc: "Admin commands for the ANILIST module.",
		},
		GroupDef {
			name: "general",
			desc: "Bot configuration configuration for admin only.",
		},
	],
});
