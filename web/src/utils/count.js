export const prettyQuoteNum = (num) => {
    if (num >= 10000) {
        return (num / 10000).toFixed(1) + '万';
    }
    else if (num >= 1000) {
        return (num / 1000).toFixed(1) + '千';
    }
    return num;
};
//# sourceMappingURL=count.js.map