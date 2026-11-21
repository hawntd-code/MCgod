# 05 · Guía de redacción

Regla para todo texto del proyecto: la interfaz, la documentación, los commits, la base de conocimiento y las respuestas del asistente.

## La regla

**Escribe lo que la cosa hace, con verbos y sustantivos concretos.** Si una frase podría estar en el folleto de cualquier otro producto, no sirve.

| En vez de | Escribe |
|---|---|
| Potencia tu creatividad con herramientas robustas | Crea ítems, bloques y criaturas sin escribir Java |
| Una experiencia fluida y sin fisuras | Abre en 2 segundos y usa la mitad de memoria *(solo si está medido)* |
| Es importante destacar que se necesita Java | Necesitas Java 21 para compilar mods |
| Lleva tus mods al siguiente nivel | Agrega procedures para que tus bloques reaccionen al jugador |
| Explora una amplia gama de opciones | 37 tipos de elemento |

## Lo que está prohibido

1. **Muletillas de relleno:** "cabe destacar", "es importante mencionar", "vale la pena señalar", "en este sentido", "dicho esto".
2. **Cierres de resumen:** "en resumen", "en conclusión", "en definitiva". Un texto claro no necesita repetirse.
3. **Introducciones genéricas:** "en el mundo actual", "en la era digital", "en un mundo cada vez más…".
4. **Metáforas de IA:** viajes, inmersiones ("sumérgete"), tapices, corazones, piedras angulares, navegar por complejidades.
5. **Promesas de folleto:** "de primera clase", "de última generación", "llevar al siguiente nivel", "desbloquear tu potencial", "sin fisuras", "con solo unos clics".
6. **Verbos vagos:** potenciar, impulsar, revolucionar, empoderar, optimizar (sin decir qué).
7. **Adjetivos inflados:** robusto, potente, intuitivo, crucial, meticuloso, fascinante, integral.
8. **Fórmulas:** "No es X, es Y". "No solo X, sino también Y" repetido.
9. **Fórmulas serviles en el chat:** "¡Claro!", "¡Excelente pregunta!", "Espero que esto te ayude", "No dudes en preguntar".

La lista completa, en español e inglés, está en [`style/frases-prohibidas.json`](../style/frases-prohibidas.json).

## También evita

- Tríadas por reflejo ("rápido, simple y potente"). Usa la cantidad de elementos que de verdad hay.
- Pares de adjetivos calcados del inglés ("fácil y sencillo").
- Gerundios encadenados ("permitiendo", "logrando", "siendo").
- Exceso de rayas (—) para insertar aclaraciones; usa comas, dos puntos o paréntesis.
- Emoji como viñetas o como íconos.
- Mayúsculas en cada palabra de un título en español. Se escribe "Base de conocimiento", no "Base De Conocimiento".
- Cifras inventadas. Un número aparece solo si se midió o si sale del código.

## Cómo se comprueba

La prueba `app/src/__tests__/redaccion.test.ts` revisa todos los textos de la interfaz (español e inglés), el README y la documentación contra la lista. Si un cambio agrega una frase prohibida, la integración continua falla y señala el archivo y la frase.

Esta guía es la única excepción: la prueba no la revisa porque cita las frases como ejemplo.

## Fuentes

Las listas de "palabras de IA" no sirven para detectar si un texto lo escribió una máquina ([Originality.ai](https://originality.ai/blog/can-humans-detect-chatgpt) lo comprobó con datos), pero sí funcionan como filtro de estilo. Partimos de [Grammarly](https://www.grammarly.com/blog/ai-writing/common-ai-words/), [Plus AI](https://plusai.com/blog/the-most-overused-chatgpt-words), [Genbeta](https://www.genbeta.com/inteligencia-artificial/estas-palabras-frases-tus-textos-dejan-claro-has-usado-chatgpt/amp), [Applesfera](https://www.applesfera.com/curiosidades/te-han-pillado-palabras-frases-que-hacen-evidente-que-utilizaste-chatgpt-lugar-pensar-a-apple) y el estudio de la Universidad Complutense citado por [Teleamazonas](https://www.teleamazonas.com/tendencias/entretenimiento/tecnologia/efecto-chatgpt-estudios-senialan-empezamos-hablar-robots-105120/).
