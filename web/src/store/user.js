import { defineStore } from "pinia";
import { computed, ref } from "vue";
/** 本地存储的用户令牌键名 */
export const TOKEN_KEY = 'EVT_TOKEN';
export const useStoreUser = defineStore('user', () => {
    const userInfo = ref({
        id: 0,
        username: '',
        nickname: '',
        created_on: 0,
        follows: 0,
        followings: 0,
        tweets_count: 0,
        is_admin: false,
    });
    const userLogined = computed(() => userInfo.value.id > 0);
    function updateUserinfo(data) {
        userInfo.value = data;
    }
    function userLogout() {
        localStorage.removeItem(TOKEN_KEY);
        userInfo.value = {
            id: 0,
            nickname: '',
            username: '',
            created_on: 0,
            follows: 0,
            followings: 0,
            tweets_count: 0,
            is_admin: false,
        };
    }
    return {
        userInfo,
        userLogined,
        updateUserinfo, userLogout,
    };
});
//# sourceMappingURL=user.js.map