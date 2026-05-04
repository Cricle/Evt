import { describe, expect, it } from 'vitest';
import { goToAuth } from '@/utils/authRoute';
describe('auth route utils', () => {
    it('pushes the auth route with mode and redirect query', async () => {
        let pushed;
        const router = {
            push(payload) {
                pushed = payload;
                return Promise.resolve(payload);
            },
        };
        await goToAuth(router, 'signup', '/#/compose?space=public');
        expect(pushed).toEqual({
            name: 'auth',
            query: {
                mode: 'signup',
                redirect: '/#/compose?space=public',
            },
        });
    });
    it('omits the redirect query when it is not provided', async () => {
        let pushed;
        const router = {
            push(payload) {
                pushed = payload;
                return Promise.resolve(payload);
            },
        };
        await goToAuth(router, 'signin');
        expect(pushed).toEqual({
            name: 'auth',
            query: {
                mode: 'signin',
            },
        });
    });
});
//# sourceMappingURL=authRoute.test.js.map