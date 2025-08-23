//! 流式响应处理模块
use crate::{
    error::{NanoError, Result},
    types::StreamCompletionResponse,
};
use async_stream::try_stream;
use bytes::{Bytes, BytesMut};
use futures::{Stream, StreamExt};
use log::debug;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

// ================================================================================================
// 流式响应包装器
// ================================================================================================

const DONE_CHUNK: &str = "[DONE]";

/// 一个无状态的流处理器，用于解析 SSE (Server-Sent Events) 数据流
#[derive(Debug, Clone, Default)]
pub struct StreamWrapper;

impl StreamWrapper {
    /// 创建一个新的 `StreamWrapper` 实例
    ///
    /// 这是一个无状态的结构体，所以 `new` 只是 `default` 的别名
    pub fn new() -> Self {
        StreamWrapper
    }

    /// 将一个 `BytesStream` 转换为一个解析 `StreamCompletionResponse` 的流
    pub fn stream<S>(
        &self,
        mut bytes_stream: S,
    ) -> impl Stream<Item = Result<StreamCompletionResponse>>
    where
        S: Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send + 'static + Unpin,
    {
        try_stream! {
            let mut buffer = BytesMut::new();
            while let Some(bytes_res) = bytes_stream.next().await {
                let bytes = bytes_res.map_err(NanoError::from)?;
                buffer.extend_from_slice(&bytes);

                loop {
                    if let Some(pos) = buffer.windows(2).position(|w| w == [b'\n', b'\n']) {
                        let event_bytes = buffer.split_to(pos + 2);

                        let event_str = String::from_utf8_lossy(&event_bytes).to_string();

                        let mut data = String::new();
                        for line in event_str.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with(':') {
                                continue;
                            }
                            if let Some(content) = trimmed.strip_prefix("data: ") {
                                if !data.is_empty() {
                                    data.push('\n');
                                }
                                data.push_str(content);
                            }
                        }

                        if !data.is_empty() && data != DONE_CHUNK {
                            match serde_json::from_str(&data) {
                                Ok(resp) => yield resp,
                                Err(e) => Err(NanoError::Json(format!("Failed to parse event: '{}', error: {}", data, e)))?,
                            }
                        }
                    } else {
                        break;
                    }
                }
            }

            if !buffer.is_empty() {
                debug!("Leftover buffer: {:?}", String::from_utf8_lossy(&buffer));
            }
        }
    }

    // process_chunk 已弃用，使用状态流处理
}

/// `Stream<Item = Result<StreamCompletionResponse>>` 的简单包装
pub struct CompletionStream {
    inner: Pin<Box<dyn Stream<Item = Result<StreamCompletionResponse>> + Send>>,
}

impl CompletionStream {
    /// 创建一个新的 `CompletionStream`
    pub fn new(stream: impl Stream<Item = Result<StreamCompletionResponse>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl Stream for CompletionStream {
    type Item = Result<StreamCompletionResponse>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}