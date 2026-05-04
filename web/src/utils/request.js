import axios from 'axios';
import { TOKEN_KEY } from '@/store/user';
import { apiBaseUrl } from '@/utils/api';
const service = axios.create({
    baseURL: apiBaseUrl,
    timeout: 30000,
});
service.interceptors.request.use((config) => {
    // 鉴权Header
    if (localStorage.getItem(TOKEN_KEY)) {
        config.headers['Authorization'] =
            'Bearer ' + localStorage.getItem(TOKEN_KEY);
    }
    return config;
}, (error) => {
    return Promise.reject(error);
});
service.interceptors.response.use((response) => {
    const { data = {}, code = 0 } = response?.data || {};
    if (+code === 0) {
        return data || {};
    }
    else {
        return Promise.reject(response?.data || {});
    }
}, (error = {}) => {
    const { response = {} } = error || {};
    // 重定向
    if (+response?.status === 401) {
        localStorage.removeItem(TOKEN_KEY);
        if (response?.data.code !== 10005) {
            window.$message.warning(response?.data.msg || '鉴权失败');
        }
        else {
            const redirect = encodeURIComponent(window.location.hash || '/');
            window.location.assign(`/#/auth?mode=signin&redirect=${redirect}`);
        }
    }
    else {
        window.$message.error(response?.data?.msg || '请求失败');
    }
    return Promise.reject(response?.data || {});
});
export default service;
export function request(config) {
    return service(config);
}
/**
 * 创建一个API对象，支持链式调用
 */
export function createApi() {
    const createProxy = (...names) => new Proxy((...args) => {
        let _path = [];
        let method = '';
        let methods = ['get', 'post'];
        for (const name of names) {
            const lowerName = name.toLowerCase();
            if (methods.includes(lowerName) && !method) {
                method = lowerName;
            }
            else {
                _path.push(name);
            }
        }
        if (!method)
            method = 'get';
        // 如果最后一条路径是 _self 则代表不需要它，直接去掉它
        if (_path[_path.length - 1] === "_self")
            _path.pop();
        return request({
            method,
            url: _path.join('/'),
            ...(method === 'get' ? { params: args[0] } : { data: args[0] }),
        });
    }, {
        get(target, p) {
            if (p === 'then')
                return undefined;
            if (!target[p])
                target[p] = createProxy(...names, p);
            return target[p];
        },
    });
    return createProxy();
}
export const Api = createApi();
//# sourceMappingURL=request.js.map