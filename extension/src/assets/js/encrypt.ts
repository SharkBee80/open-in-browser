import * as jose from "jose";

// 1. 定义 Payload 类型
interface Payload extends Record<string, any> {
  iat: number;
  exp: number;
}

export const encodeKey = (secret: string): Uint8Array<ArrayBufferLike> => {
  return new TextEncoder().encode(secret);
};

/**
 * 签发 Token (生成)
 */
export const signToken = async (
  payload: Payload,
  key: Uint8Array<ArrayBufferLike> | jose.KeyObject | CryptoKey | jose.JWK,
  header: jose.JWTHeaderParameters = { alg: "HS256", typ: "JWT" },
): Promise<string> => {
  return await new jose.SignJWT(payload).setProtectedHeader(header).sign(key);
};

/**
 * 校验 Token (验证)
 */
export const verifyToken = async (
  token: string,
  key: jose.CryptoKey | jose.KeyObject | jose.JWK | Uint8Array,
  options?: jose.JWTVerifyOptions,
): Promise<Payload | null> => {
  try {
    // 强制转换为 UserPayload 类型
    const { payload } = await jose.jwtVerify(token, key, options);
    return payload as Payload;
  } catch (error) {
    console.error("Token 校验失败:", error);
    return null;
  }
};

/**
 * 解析 Token (仅解码，不校验合法性)
 */
export const decodeToken = (token: string): Payload | null => {
  try {
    return jose.decodeJwt(token) as Payload;
  } catch (error) {
    console.error("Token 解码失败:", error);
    return null;
  }
};
