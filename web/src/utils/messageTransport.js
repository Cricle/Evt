import { Api } from '@/utils/request';
import { isTauriRuntime } from '@/utils/platform';
import { TOKEN_KEY } from '@/store/user';
function tauriInvoke() {
    if (!isTauriRuntime() || typeof window === 'undefined') {
        return null;
    }
    const tauriWindow = window;
    return tauriWindow.__TAURI__?.invoke ?? tauriWindow.__TAURI_INTERNALS__?.invoke ?? null;
}
function bearerToken() {
    return localStorage.getItem(TOKEN_KEY) ?? '';
}
async function grpcListMessages(params) {
    const invoke = tauriInvoke();
    if (!invoke) {
        throw new Error('tauri grpc transport unavailable');
    }
    return invoke('grpc_list_legacy_messages', {
        bearerToken: bearerToken(),
        style: params.style,
        page: params.page,
        pageSize: params.page_size,
    });
}
async function grpcUnreadCount() {
    const invoke = tauriInvoke();
    if (!invoke) {
        throw new Error('tauri grpc transport unavailable');
    }
    return invoke('grpc_legacy_unread_count', {
        bearerToken: bearerToken(),
    });
}
async function grpcReadMessage(id) {
    const invoke = tauriInvoke();
    if (!invoke) {
        throw new Error('tauri grpc transport unavailable');
    }
    await invoke('grpc_mark_message_read', {
        bearerToken: bearerToken(),
        messageId: id,
    });
}
async function grpcReadAll() {
    const invoke = tauriInvoke();
    if (!invoke) {
        throw new Error('tauri grpc transport unavailable');
    }
    await invoke('grpc_mark_all_messages_read', {
        bearerToken: bearerToken(),
    });
}
async function grpcSendWhisper(data) {
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
export async function listLegacyMessages(params) {
    if (isTauriRuntime()) {
        try {
            return await grpcListMessages(params);
        }
        catch (_error) {
            return Api.v1.user.get.messages(params);
        }
    }
    return Api.v1.user.get.messages(params);
}
export async function unreadLegacyMessageCount() {
    if (isTauriRuntime()) {
        try {
            return await grpcUnreadCount();
        }
        catch (_error) {
            return Api.v1.user.get.msgcount.unread({});
        }
    }
    return Api.v1.user.get.msgcount.unread({});
}
export async function markLegacyMessageRead(id) {
    if (isTauriRuntime()) {
        try {
            await grpcReadMessage(id);
            return;
        }
        catch (_error) {
            // fall through
        }
    }
    await Api.v1.user.message.post.read({ id });
}
export async function markAllLegacyMessagesRead() {
    if (isTauriRuntime()) {
        try {
            await grpcReadAll();
            return;
        }
        catch (_error) {
            // fall through
        }
    }
    await Api.v1.user.message.post.readall();
}
export async function sendLegacyWhisper(data) {
    if (isTauriRuntime()) {
        try {
            await grpcSendWhisper(data);
            return;
        }
        catch (_error) {
            // fall through
        }
    }
    await Api.v1.user.post.whisper(data);
}
//# sourceMappingURL=messageTransport.js.map