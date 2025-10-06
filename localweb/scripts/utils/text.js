/**
 * Text utility functions
 */

/**
 * Escapes HTML special characters to prevent XSS attacks
 * @param {string} text - The text to escape
 * @returns {string} - The escaped HTML text
 */
export function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

/**
 * Truncates text to a specified length and adds ellipsis
 * @param {string} text - The text to truncate
 * @param {number} maxLength - Maximum length before truncation
 * @returns {string} - The truncated text with ellipsis if needed
 */
export function truncateText(text, maxLength = 50) {
    if (!text || text.length <= maxLength) {
        return text;
    }
    return text.substring(0, maxLength) + '...';
}

/**
 * Capitalizes the first letter of a string
 * @param {string} text - The text to capitalize
 * @returns {string} - The text with first letter capitalized
 */
export function capitalizeFirst(text) {
    if (!text) return text;
    return text.charAt(0).toUpperCase() + text.slice(1);
}

/**
 * Get initials from a name
 * @param {string} name - The name to get initials from
 * @returns {string} - The initials
 */
export function getInitials(name, maxLength = 1) {
    if (!name) return '';
    return name.trim()
        .split(' ')
        .map(word => word.charAt(0).toUpperCase())
        .slice(0, maxLength) // Take only first initial
        .join('');
}