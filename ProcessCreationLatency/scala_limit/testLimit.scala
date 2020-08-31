import akka.actor.Actor
import akka.actor.ActorSystem
import akka.actor.ActorRef
import akka.actor.Props
import scala.collection.mutable.ListBuffer
import java.io._

class HelloActor extends Actor {
  def receive = {
    case "hello" => sender ! "hello"
  }
}

object Main extends App {
  val pw = new PrintWriter(new File("test_scala_limit.txt" ))
  val system = ActorSystem("HelloSystem")

  var numberOfActors:Int = 1
  while(true){
    system.actorOf(Props[HelloActor])
    val timestamp: Long = System.currentTimeMillis / 1000
    numberOfActors = numberOfActors + 1
    pw.write("ts(sec): " + timestamp + " actor no: " + numberOfActors + "\n")
  }
  pw.close
  system.terminate()
}
