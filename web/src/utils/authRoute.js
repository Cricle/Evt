export function goToAuth(router, mode = 'signin', redirect) {
    return router.push({
        name: 'auth',
        query: {
            mode,
            ...(redirect ? { redirect } : {}),
        },
    });
}
//# sourceMappingURL=authRoute.js.map