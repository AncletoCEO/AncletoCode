export const APP_TITLE = "AncletoCode";
export const THURSDAY_TITLE = `${APP_TITLE} - Feliz Jueves 🏃`;

export function titleForDate(date: Date): string {
  return date.getDay() === 4 ? THURSDAY_TITLE : APP_TITLE;
}
