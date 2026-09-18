import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArticleNyTimesIcon` instead.
 *
 * @example
 * ```svelte
 * <ArticleNyTimes color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArticleNyTimes: Component<IconComponentProps, {}, "">;
type ArticleNyTimes = ReturnType<typeof ArticleNyTimes>;
export default ArticleNyTimes;
