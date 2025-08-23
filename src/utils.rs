//! 工具函数模块
use crate::types::{Message, Role};

/// 创建消息的便捷函数
///
/// # 参数
///
/// * `role` - 消息角色
/// * `content` - 消息内容
///
/// # 返回
///
/// 新创建的消息实例
pub fn message(role: Role, content: &str) -> Message {
    Message {
        role,
        content: content.to_string(),
    }
}

/// 准备发送到 API 的消息列表
///
/// 如果系统消息不为空，则将其作为第一条消息。
/// 准备发送到 API 的消息列表
/// 
/// 如果系统消息不为空，则将其作为第一条消息。
pub(crate) fn prepare_messages(system_message: &str, messages: &[Message]) -> Vec<Message> {
    let system_iter = if !system_message.is_empty() {
        vec![message(Role::System, system_message)].into_iter()
    } else {
        vec![].into_iter()
    };
    system_iter.chain(messages.iter().cloned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Role;

    #[test]
    fn test_message_creation() {
        let msg = message(Role::User, "Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_prepare_messages_with_system_message() {
        let system_message = "You are a helpful assistant.";
        let messages = vec![message(Role::User, "Hello")];
        let prepared = prepare_messages(system_message, &messages);
        assert_eq!(prepared.len(), 2);
        assert_eq!(prepared[0].role, Role::System);
        assert_eq!(prepared[0].content, system_message);
        assert_eq!(prepared[1].role, Role::User);
    }

    #[test]
    fn test_prepare_messages_without_system_message() {
        let system_message = "";
        let messages = vec![message(Role::User, "Hello")];
        let prepared = prepare_messages(system_message, &messages);
        assert_eq!(prepared.len(), 1);
        assert_eq!(prepared[0].role, Role::User);
    }
}