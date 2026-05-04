import DOMPurify from 'dompurify';
import { PostItemTypeEnum } from '@/utils/IEnum';
import { toPlainText } from '@/utils/content';
const TEXT_ALLOWED_TAGS = ['p', 'br', 'strong', 'em', 'ul', 'ol', 'li', 'blockquote', 'code', 'pre'];
const TEXT_ALLOWED_ATTR = [];
const removeMediaNodes = (html) => {
    if (!html) {
        return '';
    }
    const doc = new DOMParser().parseFromString(html, 'text/html');
    doc.querySelectorAll('img, figure.image, figure.image-style-block, figure.image-style-side, video, iframe, oembed').forEach((node) => {
        node.remove();
    });
    return doc.body.innerHTML;
};
export const sanitizeEditorText = (html) => {
    return DOMPurify.sanitize(removeMediaNodes(html), {
        ALLOWED_TAGS: TEXT_ALLOWED_TAGS,
        ALLOWED_ATTR: TEXT_ALLOWED_ATTR,
    }).trim();
};
export const extractEditorImageUrls = (html) => {
    if (!html) {
        return [];
    }
    const doc = new DOMParser().parseFromString(html, 'text/html');
    const unique = new Set();
    const urls = [];
    doc.querySelectorAll('img').forEach((node) => {
        const src = node.getAttribute('src')?.trim();
        if (!src || unique.has(src)) {
            return;
        }
        unique.add(src);
        urls.push(src);
    });
    return urls;
};
export const syncImageAssetsWithEditor = (html, assets) => {
    const imageUrls = extractEditorImageUrls(html);
    const assetMap = new Map(assets.map((asset) => [asset.content, asset]));
    return imageUrls.map((url, index) => {
        const current = assetMap.get(url);
        return (current ?? {
            id: `editor-image-${index}-${url}`,
            kind: 'image',
            name: `图片 ${index + 1}`,
            content: url,
        });
    });
};
export const hasComposeContent = (textHtml, assets) => {
    return (toPlainText(sanitizeEditorText(textHtml)).length > 0 ||
        (assets.images?.length ?? 0) > 0 ||
        (assets.videos?.length ?? 0) > 0 ||
        (assets.attachments?.length ?? 0) > 0 ||
        (assets.links?.length ?? 0) > 0);
};
export const buildComposePostContents = (input) => {
    const contents = [];
    const textContent = sanitizeEditorText(input.textHtml);
    let sort = 100;
    if (textContent) {
        contents.push({
            content: textContent,
            type: PostItemTypeEnum.TEXT,
            sort,
        });
    }
    for (const image of input.images) {
        sort += 1;
        contents.push({
            content: image.content,
            type: PostItemTypeEnum.IMAGEURL,
            sort,
        });
    }
    for (const video of input.videos) {
        sort += 1;
        contents.push({
            content: video.content,
            type: PostItemTypeEnum.VIDEOURL,
            sort,
        });
    }
    const attachmentType = input.attachmentPrice > 0 ? PostItemTypeEnum.CHARGEATTACHMENT : PostItemTypeEnum.ATTACHMENT;
    for (const attachment of input.attachments) {
        sort += 1;
        contents.push({
            content: attachment.content,
            type: attachmentType,
            sort,
        });
    }
    for (const link of input.links) {
        sort += 1;
        contents.push({
            content: link.content,
            type: PostItemTypeEnum.LINKURL,
            sort,
        });
    }
    return {
        contents,
        plainText: toPlainText(textContent),
        textContent,
    };
};
//# sourceMappingURL=compose-editor-content.js.map