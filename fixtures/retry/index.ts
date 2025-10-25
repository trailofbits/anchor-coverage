export async function retry<T>(f: () => Promise<T>): Promise<T> {
  while (true) {
    try {
      return await f();
    } catch (error) {
      console.error(error);
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
}
