import { ButtonView, FileDialogButtonView, FileRepository, Plugin, } from 'ckeditor5';
import { buildApiUrl } from '@/utils/api';
import { TOKEN_KEY } from '@/store/user';
export const EVT_UPLOAD_PLUGIN_OPTIONS = 'evtUploadPluginOptions';
const uploadEndpoint = buildApiUrl('/v1/attachment');
export const normalizeUploadedAssetUrl = (value) => {
    const normalized = value.trim();
    if (!normalized) {
        return normalized;
    }
    if (/^(https?:)?\/\//i.test(normalized) || normalized.startsWith('data:') || normalized.startsWith('blob:')) {
        return normalized;
    }
    return buildApiUrl(normalized.startsWith('/') ? normalized : `/${normalized}`);
};
export const resolveUploadAuthHeader = () => `Bearer ${localStorage.getItem(TOKEN_KEY) || ''}`;
export const normalizeLinkInput = (value) => value.trim();
export const uploadFile = async (file, kind) => {
    const form = new FormData();
    form.append('type', kind);
    form.append('file', file);
    const response = await fetch(uploadEndpoint, {
        method: 'POST',
        headers: {
            Authorization: resolveUploadAuthHeader(),
        },
        body: form,
    });
    if (!response.ok) {
        throw new Error(`upload failed: ${response.status}`);
    }
    const payload = await response.json();
    const content = normalizeUploadedAssetUrl(payload?.data?.content ?? '');
    if (!content) {
        throw new Error('invalid upload response');
    }
    return {
        kind,
        content,
        name: file.name,
    };
};
export const uploadFilesSequentially = async (files, kind, options, upload = uploadFile) => {
    for (const file of files) {
        options.onStart?.(file.name, kind);
        try {
            const asset = await upload(file, kind);
            options.onUploaded?.(asset);
        }
        catch (error) {
            options.onError?.(error, file.name, kind);
        }
        finally {
            options.onFinish?.(file.name, kind);
        }
    }
};
export class EvtImageUploadAdapter {
    loader;
    options;
    uploadRequest;
    constructor(loader, options, uploadRequest = uploadFile) {
        this.loader = loader;
        this.options = options;
        this.uploadRequest = uploadRequest;
    }
    async upload() {
        const file = await this.loader.file;
        if (!file) {
            throw new Error('missing upload file');
        }
        this.options.onStart?.(file.name, 'public/image');
        try {
            const asset = await this.uploadRequest(file, 'public/image');
            this.options.onUploaded?.(asset);
            return { default: asset.content };
        }
        catch (error) {
            this.options.onError?.(error, file.name, 'public/image');
            throw error;
        }
        finally {
            this.options.onFinish?.(file.name, 'public/image');
        }
    }
    abort() { }
}
const getOptions = (editor) => {
    return (editor.config.get(EVT_UPLOAD_PLUGIN_OPTIONS) ?? {});
};
export default class EvtUploadPlugin extends Plugin {
    static get requires() {
        return [FileRepository];
    }
    static get pluginName() {
        return 'EvtUploadPlugin';
    }
    init() {
        const editor = this.editor;
        const options = getOptions(editor);
        const fileRepository = editor.plugins.get(FileRepository);
        fileRepository.createUploadAdapter = (loader) => {
            return new EvtImageUploadAdapter(loader, options);
        };
        editor.ui.componentFactory.add('evtVideoUpload', () => {
            const view = new FileDialogButtonView(editor.locale);
            view.set({
                acceptedType: 'video/*',
                allowMultipleFiles: true,
                label: '视频',
                tooltip: true,
            });
            view.on('done', (_evt, files) => {
                void uploadFilesSequentially(Array.from(files || []), 'public/video', options);
            });
            return view;
        });
        editor.ui.componentFactory.add('evtAttachmentUpload', () => {
            const view = new FileDialogButtonView(editor.locale);
            view.set({
                acceptedType: '*',
                allowMultipleFiles: true,
                label: '附件',
                tooltip: true,
            });
            view.on('done', (_evt, files) => {
                void uploadFilesSequentially(Array.from(files || []), 'attachment', options);
            });
            return view;
        });
        editor.ui.componentFactory.add('evtLink', () => {
            const button = new ButtonView(editor.locale);
            button.set({
                label: '链接',
                withText: true,
                tooltip: true,
            });
            button.on('execute', () => {
                const url = window.prompt('请输入链接');
                if (!url) {
                    return;
                }
                options.onLinkCreate?.(normalizeLinkInput(url));
            });
            return button;
        });
    }
}
//# sourceMappingURL=ckeditor-upload-plugin.js.map
