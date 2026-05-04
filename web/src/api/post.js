/**
 * 这个接口暂时保留着，暂时不放进createApi里，后续如果有需要再考虑重构
 * 当然，也可以作为另一种API定义方式的示例，供后续参考
 */
import { request } from '@/utils/request';
export const buildPostReactionsPath = (postId) => `/v1/posts/${postId}/reactions`;
/** 获取动态列表 */
export const getPosts = (params) => {
    return request({
        method: 'get',
        url: '/v1/posts',
        params,
    });
};
/** 获取标签列表 */
export const getTags = (params) => {
    return request({
        method: 'get',
        url: '/v1/tags',
        params,
    });
};
/** 获取动态详情 */
export const getPost = (params) => {
    return request({
        method: 'get',
        url: '/v1/post',
        params,
    });
};
/** 获取动态评论列表 */
export const getPostComments = (params) => {
    return request({
        method: 'get',
        url: '/v1/post/comments',
        params,
    });
};
/** 获取联系人列表 */
export const getIndexTrends = (params) => {
    return request({
        method: 'get',
        url: '/v1/trends/index',
        params,
    });
};
/** 发布动态 */
export const createPost = (data) => {
    return request({
        method: 'post',
        url: '/v1/post',
        data,
    });
};
/** 删除动态 */
export const deletePost = (data) => {
    return request({
        method: 'delete',
        url: '/v1/post',
        data,
    });
};
/** 锁定/解锁动态 */
export const lockPost = (data) => {
    return request({
        method: 'post',
        url: '/v1/post/lock',
        data,
    });
};
/** 置顶/取消置顶动态 */
export const stickPost = (data) => {
    return request({
        method: 'post',
        url: '/v1/post/stick',
        data,
    });
};
/** 设为亮点/取消亮点动态 */
export const highlightPost = (data) => {
    return request({
        method: 'post',
        url: '/v1/post/highlight',
        data,
    });
};
/** 置顶/取消置顶动态 */
export const visibilityPost = (data) => {
    return request({
        method: 'post',
        url: '/v1/post/visibility',
        data,
    });
};
/** 发布动态评论 */
export const createComment = (data) => {
    return request({
        method: 'post',
        url: '/v1/post/comment',
        data,
    });
};
/** 删除评论 */
export const deleteComment = (data) => {
    return request({
        method: 'delete',
        url: '/v1/post/comment',
        data,
    });
};
/** 精选评论 */
export const highlightComment = (data) => {
    return request({
        method: 'post',
        url: '/v1/post/comment/highlight',
        data,
    });
};
export const getPostReactions = (postId) => {
    return request({
        method: 'get',
        url: buildPostReactionsPath(postId),
    });
};
export const togglePostReaction = (postId, emoji) => {
    return request({
        method: 'post',
        url: buildPostReactionsPath(postId),
        data: {
            emoji,
        },
    });
};
/** 置顶/取消置顶话题 */
export const stickTopic = (data) => {
    return request({
        method: 'post',
        url: '/v1/topic/stick',
        data,
    });
};
/** 置顶/取消置顶话题 */
export const pinTopic = (data) => {
    return request({
        method: 'post',
        url: '/v1/topic/pin',
        data,
    });
};
/** 关注话题 */
export const followTopic = (data) => {
    return request({
        method: 'post',
        url: '/v1/topic/follow',
        data,
    });
};
/** 取消关注话题 */
export const unfollowTopic = (data) => {
    return request({
        method: 'post',
        url: '/v1/topic/unfollow',
        data,
    });
};
//# sourceMappingURL=post.js.map