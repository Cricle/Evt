import { describe, expect, it } from 'vitest';
import { buildComposePostContents, hasComposeContent, sanitizeEditorText, syncImageAssetsWithEditor, } from '@/components/compose-editor-content';
import { PostItemTypeEnum } from '@/utils/IEnum';
describe('compose editor content helpers', () => {
    it('removes embedded media from text content', () => {
        expect(sanitizeEditorText('<p>hello</p><figure class="image"><img src="/v1/attachments/1"></figure><p>world</p>')).toBe('<p>hello</p><p>world</p>');
    });
    it('keeps image assets aligned with editor html', () => {
        const assets = [
            {
                id: 'img-1',
                kind: 'image',
                name: 'cover.png',
                content: '/v1/attachments/1',
            },
            {
                id: 'img-2',
                kind: 'image',
                name: 'extra.png',
                content: '/v1/attachments/2',
            },
        ];
        expect(syncImageAssetsWithEditor('<figure class="image"><img src="/v1/attachments/2"></figure><figure class="image"><img src="/v1/attachments/3"></figure>', assets)).toEqual([
            {
                id: 'img-2',
                kind: 'image',
                name: 'extra.png',
                content: '/v1/attachments/2',
            },
            {
                id: 'editor-image-1-/v1/attachments/3',
                kind: 'image',
                name: '图片 2',
                content: '/v1/attachments/3',
            },
        ]);
    });
    it('treats media-only drafts as valid compose content', () => {
        expect(hasComposeContent('', {
            images: [],
            videos: [],
            attachments: [
                {
                    id: 'file-1',
                    kind: 'attachment',
                    name: 'demo.zip',
                    content: '/v1/attachments/9',
                },
            ],
            links: [],
        })).toBe(true);
    });
    it('builds legacy contents without changing api structure', () => {
        const result = buildComposePostContents({
            textHtml: '<p>hello #evt</p><figure class="image"><img src="/v1/attachments/1"></figure>',
            images: [
                {
                    id: 'img-1',
                    kind: 'image',
                    name: 'cover.png',
                    content: '/v1/attachments/1',
                },
            ],
            videos: [
                {
                    id: 'video-1',
                    kind: 'video',
                    name: 'demo.mp4',
                    content: '/v1/attachments/2',
                },
            ],
            attachments: [
                {
                    id: 'attachment-1',
                    kind: 'attachment',
                    name: 'demo.zip',
                    content: '/v1/attachments/3',
                },
            ],
            links: [
                {
                    id: 'link-1',
                    kind: 'link',
                    name: 'Evt',
                    content: 'https://evt.example',
                },
            ],
            attachmentPrice: 500,
        });
        expect(result.plainText).toBe('hello #evt');
        expect(result.contents).toEqual([
            { content: '<p>hello #evt</p>', type: PostItemTypeEnum.TEXT, sort: 100 },
            { content: '/v1/attachments/1', type: PostItemTypeEnum.IMAGEURL, sort: 101 },
            { content: '/v1/attachments/2', type: PostItemTypeEnum.VIDEOURL, sort: 102 },
            { content: '/v1/attachments/3', type: PostItemTypeEnum.CHARGEATTACHMENT, sort: 103 },
            { content: 'https://evt.example', type: PostItemTypeEnum.LINKURL, sort: 104 },
        ]);
    });
});
//# sourceMappingURL=compose_editor_content.test.js.map