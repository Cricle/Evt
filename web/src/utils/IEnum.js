/** 动态内容类型枚举 */
export var PostItemTypeEnum;
(function (PostItemTypeEnum) {
    /** 标题 */
    PostItemTypeEnum[PostItemTypeEnum["TITLE"] = 1] = "TITLE";
    /** 文字段落 */
    PostItemTypeEnum[PostItemTypeEnum["TEXT"] = 2] = "TEXT";
    /** 图片地址 */
    PostItemTypeEnum[PostItemTypeEnum["IMAGEURL"] = 3] = "IMAGEURL";
    /** 视频地址 */
    PostItemTypeEnum[PostItemTypeEnum["VIDEOURL"] = 4] = "VIDEOURL";
    /** 音频地址 */
    PostItemTypeEnum[PostItemTypeEnum["AUDIOURL"] = 5] = "AUDIOURL";
    /** 链接地址 */
    PostItemTypeEnum[PostItemTypeEnum["LINKURL"] = 6] = "LINKURL";
    /** 附件资源 */
    PostItemTypeEnum[PostItemTypeEnum["ATTACHMENT"] = 7] = "ATTACHMENT";
    /** 收费资源 */
    PostItemTypeEnum[PostItemTypeEnum["CHARGEATTACHMENT"] = 8] = "CHARGEATTACHMENT";
})(PostItemTypeEnum || (PostItemTypeEnum = {}));
/** 回复内容类型枚举 */
export var CommentItemTypeEnum;
(function (CommentItemTypeEnum) {
    /** 标题 */
    CommentItemTypeEnum[CommentItemTypeEnum["TITLE"] = 1] = "TITLE";
    /** 文字段落 */
    CommentItemTypeEnum[CommentItemTypeEnum["TEXT"] = 2] = "TEXT";
    /** 图片地址 */
    CommentItemTypeEnum[CommentItemTypeEnum["IMAGEURL"] = 3] = "IMAGEURL";
    /** 视频地址 */
    CommentItemTypeEnum[CommentItemTypeEnum["VIDEOURL"] = 4] = "VIDEOURL";
    /** 音频地址 */
    CommentItemTypeEnum[CommentItemTypeEnum["AUDIOURL"] = 5] = "AUDIOURL";
    /** 链接地址 */
    CommentItemTypeEnum[CommentItemTypeEnum["LINKURL"] = 6] = "LINKURL";
})(CommentItemTypeEnum || (CommentItemTypeEnum = {}));
/** 附件类型枚举 */
export var AttachmentTypeEnum;
(function (AttachmentTypeEnum) {
    /** 图片 */
    AttachmentTypeEnum[AttachmentTypeEnum["IMAGE"] = 1] = "IMAGE";
    /** 视频 */
    AttachmentTypeEnum[AttachmentTypeEnum["VIDEO"] = 2] = "VIDEO";
    /** 其他 */
    AttachmentTypeEnum[AttachmentTypeEnum["OTHER"] = 3] = "OTHER";
})(AttachmentTypeEnum || (AttachmentTypeEnum = {}));
/** 消息类型枚举 */
export var MessageTypeEnum;
(function (MessageTypeEnum) {
    /** 动态 */
    MessageTypeEnum[MessageTypeEnum["POST"] = 1] = "POST";
    /** 评论 */
    MessageTypeEnum[MessageTypeEnum["COMMENT"] = 2] = "COMMENT";
    /** 回复 */
    MessageTypeEnum[MessageTypeEnum["REPLY"] = 3] = "REPLY";
    /** 私信 */
    MessageTypeEnum[MessageTypeEnum["PRIVATELETTER"] = 4] = "PRIVATELETTER";
    /** 添加好友申请 */
    MessageTypeEnum[MessageTypeEnum["REQUESTINGFRIEND"] = 5] = "REQUESTINGFRIEND";
    /** 系统通知 */
    MessageTypeEnum[MessageTypeEnum["SYSTEMNOTICE"] = 99] = "SYSTEMNOTICE";
})(MessageTypeEnum || (MessageTypeEnum = {}));
export var RequestingFriendStatusEnum;
(function (RequestingFriendStatusEnum) {
    /** 请求好友 */
    RequestingFriendStatusEnum[RequestingFriendStatusEnum["REQUESTING"] = 1] = "REQUESTING";
    /** 已同意好友 */
    RequestingFriendStatusEnum[RequestingFriendStatusEnum["AGREE"] = 2] = "AGREE";
    /** 已拒绝 */
    RequestingFriendStatusEnum[RequestingFriendStatusEnum["REJECT"] = 3] = "REJECT";
    /** 已删除 */
    RequestingFriendStatusEnum[RequestingFriendStatusEnum["DELETED"] = 4] = "DELETED";
})(RequestingFriendStatusEnum || (RequestingFriendStatusEnum = {}));
/** 动态可见度枚举 */
export var VisibilityEnum;
(function (VisibilityEnum) {
    /** 公开 */
    VisibilityEnum[VisibilityEnum["PUBLIC"] = 0] = "PUBLIC";
    /** 私密 */
    VisibilityEnum[VisibilityEnum["PRIVATE"] = 1] = "PRIVATE";
    /** 好友可见 */
    VisibilityEnum[VisibilityEnum["FRIEND"] = 2] = "FRIEND";
    /** 关注可见 */
    VisibilityEnum[VisibilityEnum["Following"] = 3] = "Following";
})(VisibilityEnum || (VisibilityEnum = {}));
/** 二态枚举 */
export var YesNoEnum;
(function (YesNoEnum) {
    YesNoEnum[YesNoEnum["NO"] = 0] = "NO";
    YesNoEnum[YesNoEnum["YES"] = 1] = "YES";
})(YesNoEnum || (YesNoEnum = {}));
//# sourceMappingURL=IEnum.js.map