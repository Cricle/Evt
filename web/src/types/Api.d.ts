interface Api {

    v1: {
        auth: Api.Auth.Api;
        user: Api.User.Api;
        friend: Api.Friend.Api;
        captcha: Api.Captcha.Api;
        attachment: Api.Attachment.Api;

        suggest: Api.Suggest.Api;
        admin: Api.Admin.Api;
        posts: {
            get: {
                reactions: (params: { post_id: number }) => Promise<Item.ReactionGroup[]>;
            };
            post: {
                reactions: (params: { post_id: number; emoji: string }) => Promise<{
                    active: boolean;
                    reactions: Item.ReactionGroup[];
                    comment_count: number;
                }>;
            };
        };
        spaces: {
            get: {
                _self: (params?: { limit?: number }) => Promise<Item.SpaceProps[]>;
                members: (params: { space_id: number }) => Promise<Item.SpaceMemberProps[]>;
            };
            post: {
                _self: (params: {
                    slug: string;
                    name: string;
                    description?: string;
                    visibility?: 'public' | 'private';
                }) => Promise<Item.SpaceProps>;
                members: (params: {
                    space_id: number;
                    username: string;
                    role?: 'member' | 'admin';
                }) => Promise<Item.SpaceMemberProps>;
            };
            patch: {
                members: (params: {
                    space_id: number;
                    user_id: number;
                    role?: 'member' | 'admin';
                }) => Promise<Item.SpaceMemberProps>;
            };
            delete: {
                members: (params: {
                    space_id: number;
                    user_id: number;
                }) => Promise<unknown>;
            };
        };
    }

}
