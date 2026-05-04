import { Api } from '@/utils/request';
import { isTauriRuntime } from '@/utils/platform';
import { TOKEN_KEY } from '@/store/user';

type MessageStyle = 'all' | 'system' | 'whisper' | 'requesting' | 'unread';

type LegacyMessageListResponse = Api.User.NetReq.UserGetMessages;
type LegacyUnreadCountResponse = Api.User.NetReq.UserGetUnreadMsgCount;

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

interface TauriBridge {
  invoke?: TauriInvoke;
}

interface TauriWindow extends Window {
  __TAURI__?: TauriBridge;
  __TAURI_INTERNALS__?: TauriBridge;
}

function tauriInvoke(): TauriInvoke | null {
  if (!isTauriRuntime() || typeof window === 'undefined') {
    return null;
  }

  const tauriWindow = window as TauriWindow;
  return tauriWindow.__TAURI__?.invoke ?? tauriWindow.__TAURI_INTERNALS__?.invoke ?? null;
}

function bearerToken(): string {
  return localStorage.getItem(TOKEN_KEY) ?? '';
}

async function grpcListMessages(
  params: Api.User.NetParams.UserGetMessages,
): Promise<LegacyMessageListResponse> {
  const invoke = tauriInvoke();
  if (!invoke) {
    throw new Error('tauri grpc transport unavailable');
  }

  return invoke<LegacyMessageListResponse>('grpc_list_legacy_messages', {
    bearerToken: bearerToken(),
    style: params.style,
    page: params.page,
    pageSize: params.page_size,
  });
}

async function grpcUnreadCount(): Promise<LegacyUnreadCountResponse> {
  const invoke = tauriInvoke();
  if (!invoke) {
    throw new Error('tauri grpc transport unavailable');
  }

  return invoke<LegacyUnreadCountResponse>('grpc_legacy_unread_count', {
    bearerToken: bearerToken(),
  });
}

async function grpcReadMessage(id: number): Promise<void> {
  const invoke = tauriInvoke();
  if (!invoke) {
    throw new Error('tauri grpc transport unavailable');
  }

  await invoke('grpc_mark_message_read', {
    bearerToken: bearerToken(),
    messageId: id,
  });
}

async function grpcReadAll(): Promise<void> {
  const invoke = tauriInvoke();
  if (!invoke) {
    throw new Error('tauri grpc transport unavailable');
  }

  await invoke('grpc_mark_all_messages_read', {
    bearerToken: bearerToken(),
  });
}

async function grpcSendWhisper(data: Api.User.NetParams.UserWhisper): Promise<void> {
  const invoke = tauriInvoke();
  if (!invoke) {
    throw new Error('tauri grpc transport unavailable');
  }

  await invoke('grpc_send_legacy_whisper', {
    bearerToken: bearerToken(),
    userId: data.user_id,
    content: data.content,
  });
}

export async function listLegacyMessages(
  params: Api.User.NetParams.UserGetMessages,
): Promise<LegacyMessageListResponse> {
  if (isTauriRuntime()) {
    try {
      return await grpcListMessages(params);
    } catch (_error) {
      return Api.v1.user.get.messages(params);
    }
  }

  return Api.v1.user.get.messages(params);
}

export async function unreadLegacyMessageCount(): Promise<LegacyUnreadCountResponse> {
  if (isTauriRuntime()) {
    try {
      return await grpcUnreadCount();
    } catch (_error) {
      return Api.v1.user.get.msgcount.unread({});
    }
  }

  return Api.v1.user.get.msgcount.unread({});
}

export async function markLegacyMessageRead(id: number): Promise<void> {
  if (isTauriRuntime()) {
    try {
      await grpcReadMessage(id);
      return;
    } catch (_error) {
      // fall through
    }
  }

  await Api.v1.user.message.post.read({ id });
}

export async function markAllLegacyMessagesRead(): Promise<void> {
  if (isTauriRuntime()) {
    try {
      await grpcReadAll();
      return;
    } catch (_error) {
      // fall through
    }
  }

  await Api.v1.user.message.post.readall();
}

export async function sendLegacyWhisper(
  data: Api.User.NetParams.UserWhisper,
): Promise<void> {
  if (isTauriRuntime()) {
    try {
      await grpcSendWhisper(data);
      return;
    } catch (_error) {
      // fall through
    }
  }

  await Api.v1.user.post.whisper(data);
}

export type { MessageStyle };
