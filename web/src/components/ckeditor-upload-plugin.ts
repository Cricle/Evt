import Plugin from '@ckeditor/ckeditor5-core/src/plugin';
import FileDialogButtonView from '@ckeditor/ckeditor5-ui/src/button/filedialogbuttonview';
import ButtonView from '@ckeditor/ckeditor5-ui/src/button/buttonview';
import FileRepository from '@ckeditor/ckeditor5-upload/src/filerepository';
import type { Editor } from '@ckeditor/ckeditor5-core';

import { buildApiUrl } from '@/utils/api';
import { TOKEN_KEY } from '@/store/user';

export type UploadKind = 'public/image' | 'public/video' | 'attachment';

export interface UploadedAsset {
  kind: UploadKind;
  content: string;
  name: string;
}

export interface UploadLifecycle {
  onStart?: (fileName: string, kind: UploadKind) => void;
  onUploaded?: (asset: UploadedAsset) => void;
  onFinish?: (fileName: string, kind: UploadKind) => void;
  onError?: (error: unknown, fileName: string, kind: UploadKind) => void;
}

export interface UploadPluginOptions extends UploadLifecycle {
  onLinkCreate?: (url: string) => void;
}

export const EVT_UPLOAD_PLUGIN_OPTIONS = 'evtUploadPluginOptions';

const uploadEndpoint = buildApiUrl('/v1/attachment');

export const resolveUploadAuthHeader = () => `Bearer ${localStorage.getItem(TOKEN_KEY) || ''}`;

export const normalizeLinkInput = (value: string) => value.trim();

export const uploadFile = async (file: File, kind: UploadKind) => {
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
  const content = payload?.data?.content;
  if (!content) {
    throw new Error('invalid upload response');
  }

  return {
    kind,
    content,
    name: file.name,
  } satisfies UploadedAsset;
};

export const uploadFilesSequentially = async (
  files: File[],
  kind: UploadKind,
  options: UploadPluginOptions,
  upload: typeof uploadFile = uploadFile,
) => {
  for (const file of files) {
    options.onStart?.(file.name, kind);
    try {
      const asset = await upload(file, kind);
      options.onUploaded?.(asset);
    } catch (error) {
      options.onError?.(error, file.name, kind);
    } finally {
      options.onFinish?.(file.name, kind);
    }
  }
};

export class EvtImageUploadAdapter {
  private readonly loader: any;
  private readonly options: UploadPluginOptions;
  private readonly uploadRequest: typeof uploadFile;

  constructor(loader: any, options: UploadPluginOptions, uploadRequest: typeof uploadFile = uploadFile) {
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
    } catch (error) {
      this.options.onError?.(error, file.name, 'public/image');
      throw error;
    } finally {
      this.options.onFinish?.(file.name, 'public/image');
    }
  }

  abort() {}
}

const getOptions = (editor: Editor) => {
  return ((editor.config.get(EVT_UPLOAD_PLUGIN_OPTIONS) as UploadPluginOptions | undefined) ?? {});
};

export default class EvtUploadPlugin extends Plugin {
  static get requires() {
    return [FileRepository] as const;
  }

  static get pluginName() {
    return 'EvtUploadPlugin';
  }

  init() {
    const editor = this.editor;
    const options = getOptions(editor);
    const fileRepository = editor.plugins.get(FileRepository);

    fileRepository.createUploadAdapter = (loader: any) => {
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
      view.on('done', (_evt, files: FileList) => {
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
      view.on('done', (_evt, files: FileList) => {
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
