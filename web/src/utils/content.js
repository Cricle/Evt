import DOMPurify from 'dompurify';
const tagExp = /(#|＃)([^#@\s])+?\s+?/g;
const atExp = /@([a-zA-Z0-9])+?\s+?/g;
const SAFE_HTML_TAGS = [
    'p',
    'br',
    'strong',
    'em',
    'ul',
    'ol',
    'li',
    'blockquote',
    'a',
    'code',
    'pre',
];
const SAFE_HTML_ATTRS = ['href', 'target', 'rel'];
export const sanitizeRichText = (content) => {
    return DOMPurify.sanitize(content || '', {
        ALLOWED_TAGS: SAFE_HTML_TAGS,
        ALLOWED_ATTR: SAFE_HTML_ATTRS,
    });
};
export const toPlainText = (content) => {
    return DOMPurify.sanitize(content || '', {
        ALLOWED_TAGS: [],
        ALLOWED_ATTR: [],
    })
        .replace(/&nbsp;/gi, ' ')
        .replace(/\u00a0/g, ' ')
        .replace(/\s+/g, ' ')
        .trim();
};
export const parsePostTag = (content) => {
    const tags = [];
    const users = [];
    const plainText = toPlainText(content);
    content = sanitizeRichText(content)
        .replace(tagExp, (item) => {
        tags.push(item.substr(1).trim());
        return ('<a class="hash-link" data-detail="tag:' +
            encodeURIComponent(item.substr(1).trim()) +
            '">' +
            item.trim() +
            '</a> ');
    })
        .replace(atExp, (item) => {
        users.push(item.substr(1).trim());
        return ('<a class="hash-link" data-detail="user:' +
            encodeURIComponent(item.substr(1).trim()) +
            '">' +
            item.trim() +
            '</a> ');
    });
    return {
        content,
        tags,
        users,
        plainText,
    };
};
export const preparePost = (content, foldHint, unfoldHint, maxSize, isFold = true) => {
    const plainText = toPlainText(content);
    const isEllipsis = plainText.length > maxSize;
    let displayText = plainText;
    if (isFold && isEllipsis) {
        displayText = plainText.substring(0, maxSize);
        let latestChar = displayText.charAt(maxSize - 1);
        if (latestChar == '#' || latestChar == '#' || latestChar == '@') {
            displayText = displayText.substring(0, maxSize - 1);
        }
    }
    content = displayText
        .replace(tagExp, (item) => {
        return ('<a class="hash-link" data-detail="tag:' +
            encodeURIComponent(item.substring(1).trim()) +
            '">' +
            item.trim() +
            '</a> ');
    })
        .replace(atExp, (item) => {
        return ('<a class="hash-link" data-detail="user:' +
            encodeURIComponent(item.substring(1).trim()) +
            '">' +
            item.trim() +
            '</a> ');
    });
    if (isEllipsis) {
        content =
            content.trimEnd() +
                (isFold ? '...&nbsp;' : '&nbsp;') +
                '<a class="hash-link" data-detail="post">' +
                (isFold ? foldHint : unfoldHint) +
                '</a> ';
    }
    return content;
};
//# sourceMappingURL=content.js.map