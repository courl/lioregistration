import { CalendarDate as DateValue } from "@internationalized/date"

const pad = (n: number, length: number) => n.toString().padStart(length, "0")

export default class CalendarDate {
  public year: number
  public month: number
  public day: number

  constructor({
    year,
    month,
    day,
    date,
  }: {
    year?: number
    month?: number
    day?: number
    date?: Date
  }) {
    date = date ?? new Date()
    this.year = year ?? date.getFullYear()
    this.month = month ?? date.getMonth() + 1
    this.day = day ?? date.getDate()
  }

  static parse(text: string): CalendarDate {
    const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(text)
    if (!match) {
      throw new Error(`Invalid CalendarDate string: ${text}`)
    }
    const [, year, month, day] = match
    return new CalendarDate({
      year: Number(year),
      month: Number(month),
      day: Number(day),
    })
  }

  /** Converts from the date type emitted by Nuxt UI date inputs. */
  static fromDateValue(
    date: { year: number; month: number; day: number } | null | undefined,
  ): CalendarDate | undefined {
    if (!date) return undefined
    return new CalendarDate({
      year: date.year,
      month: date.month,
      day: date.day,
    })
  }

  /** Converts to the date type used by Nuxt UI date inputs. */
  toDateValue(): DateValue {
    return new DateValue(this.year, this.month, this.day)
  }

  toJSON(): string {
    return `${pad(this.year, 4)}-${pad(this.month, 2)}-${pad(this.day, 2)}`
  }

  toString(): string {
    return this.toJSON()
  }

  getTime(): number {
    return this.getDate().getTime()
  }

  getDate(): Date {
    return new Date(this.year, this.month - 1, this.day)
  }

  addDays(days: number): CalendarDate {
    const date = this.getDate()
    date.setDate(date.getDate() + days)
    return new CalendarDate({ date })
  }

  static minmax(dates: CalendarDate[]): [CalendarDate, CalendarDate] {
    const toNumber = (x: CalendarDate) =>
      x.year * 10_000 + x.month * 100 + x.day
    if (dates.length === 0) {
      return [new CalendarDate({}), new CalendarDate({})]
    }
    const minmax = [dates[0], dates[0]] as [CalendarDate, CalendarDate]
    for (let index = 1; index < dates.length; index++) {
      const number_ = toNumber(dates[index])
      if (number_ < toNumber(minmax[0])) {
        minmax[0] = dates[index]
      }
      if (number_ > toNumber(minmax[1])) {
        minmax[1] = dates[index]
      }
    }
    return minmax
  }

  static generate(start: CalendarDate, end: CalendarDate): CalendarDate[] {
    if (end.getTime() < start.getTime()) {
      return []
    }
    const days =
      Math.round((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24)) + 1

    const range: CalendarDate[] = []
    for (let index = 0; index < days; ++index) {
      range.push(start.addDays(index))
    }
    return range
  }
}
